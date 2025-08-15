// 向量记忆 - 直觉和洞察的编码

use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::{SurrealClient, MemoryData, QueryCondition};

/// 向量记忆
pub struct VectorMemory {
    client: Arc<SurrealClient>,
    // 向量索引 - 内存中的快速搜索
    vector_index: Arc<tokio::sync::RwLock<HashMap<uuid::Uuid, Vec<f32>>>>,
    // 向量维度
    dimension: usize,
    // 聚类结果缓存
    clusters: Arc<tokio::sync::RwLock<Option<Vec<VectorCluster>>>>,
}

/// 向量记忆条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEntry {
    pub id: uuid::Uuid,
    pub memory: MemoryData,
    pub vector: Vec<f32>,
    pub dimension: usize,
    pub created_at: std::time::SystemTime,
    pub similarity_cache: HashMap<uuid::Uuid, f32>,
}

/// 向量聚类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorCluster {
    pub id: uuid::Uuid,
    pub centroid: Vec<f32>,
    pub members: Vec<uuid::Uuid>,
    pub radius: f32,
    pub created_at: std::time::SystemTime,
}

/// 相似度搜索结果
#[derive(Debug, Clone)]
pub struct SimilarityResult {
    pub memory: MemoryData,
    pub similarity_score: f32,
    pub distance: f32,
}

impl VectorMemory {
    pub async fn new(client: Arc<SurrealClient>) -> Result<Self> {
        Ok(Self { 
            client,
            vector_index: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            dimension: 512, // 默认向量维度
            clusters: Arc::new(tokio::sync::RwLock::new(None)),
        })
    }
    
    /// 存储向量数据
    pub async fn store(&self, memory: MemoryData) -> Result<()> {
        let vector = self.encode_to_vector(&memory)?;
        
        // 更新向量索引
        {
            let mut vector_index = self.vector_index.write().await;
            vector_index.insert(memory.id, vector.clone());
        }
        
        // 删除旧的聚类结果，因为数据发生了变化
        {
            let mut clusters = self.clusters.write().await;
            *clusters = None;
        }
        
        let vector_entry = VectorEntry {
            id: memory.id,
            memory: memory.clone(),
            vector: vector.clone(),
            dimension: vector.len(),
            created_at: std::time::SystemTime::now(),
            similarity_cache: HashMap::new(),
        };
        
        let vector_data = serde_json::to_value(&vector_entry)?;
        self.client.store(memory.id, vector_data).await
    }
    
    /// 查询向量数据
    pub async fn query(&self, condition: &QueryCondition) -> Result<Vec<MemoryData>> {
        // 使用客户端的查询功能
        self.client.query_memories(condition).await
    }
    
    /// 编码为向量
    fn encode_to_vector(&self, memory: &MemoryData) -> Result<Vec<f32>> {
        // 简单的TF-IDF式向量化方法
        let text = memory.content.to_string().to_lowercase();
        let words: Vec<&str> = text.split_whitespace().collect();
        
        // 创建固定维度的向量
        let mut vector = vec![0.0; self.dimension];
        
        // 简单的哈希映射
        for (i, word) in words.iter().enumerate().take(self.dimension) {
            let hash = self.simple_hash(word) % self.dimension;
            vector[hash] += 1.0;
        }
        
        // 正规化
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for value in vector.iter_mut() {
                *value /= norm;
            }
        }
        
        // 添加一些基于元数据的特征
        if let Some(features) = self.extract_metadata_features(memory) {
            for (i, feature) in features.iter().enumerate().take(10) {
                if i < vector.len() {
                    vector[i] += feature * 0.1; // 加权的元数据特征
                }
            }
        }
        
        Ok(vector)
    }
    
    fn simple_hash(&self, s: &str) -> usize {
        let mut hash = 0usize;
        for byte in s.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as usize);
        }
        hash
    }
    
    fn extract_metadata_features(&self, memory: &MemoryData) -> Option<Vec<f32>> {
        let mut features = vec![0.0; 10];
        
        // 基于数据类型的特征
        match memory.data_type {
            crate::optimized_persistence::DataType::Perception => features[0] = 1.0,
            crate::optimized_persistence::DataType::Action => features[1] = 1.0,
            crate::optimized_persistence::DataType::Conversation => features[2] = 1.0,
            crate::optimized_persistence::DataType::Knowledge => features[3] = 1.0,
            crate::optimized_persistence::DataType::Experience => features[4] = 1.0,
        }
        
        // 时间特征
        if let Ok(duration) = memory.timestamp.duration_since(std::time::UNIX_EPOCH) {
            features[5] = (duration.as_secs() % 86400) as f32 / 86400.0; // 一天内的时间
        }
        
        // 内容长度特征
        features[6] = (memory.content.to_string().len() as f32).ln();
        
        Some(features)
    }
    
    /// 向量相似度搜索
    pub async fn similarity_search(&self, query_vector: Vec<f32>, top_k: usize) -> Result<Vec<SimilarityResult>> {
        log::info!("执行向量相似度搜索, top_k: {}", top_k);
        
        let mut results = Vec::new();
        let vector_index = self.vector_index.read().await;
        
        // 获取所有记忆数据
        let all_memories = self.client.query_memories(&QueryCondition {
            session_id: None,
            data_type: None,
            time_range: None,
            keywords: vec![],
            limit: None,
        }).await?;
        
        // 计算相似度
        for memory in all_memories {
            if let Some(vector) = vector_index.get(&memory.id) {
                let similarity = self.cosine_similarity(&query_vector, vector);
                let distance = self.euclidean_distance(&query_vector, vector);
                
                results.push(SimilarityResult {
                    memory,
                    similarity_score: similarity,
                    distance,
                });
            }
        }
        
        // 按相似度排序
        results.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score)
            .unwrap_or(std::cmp::Ordering::Equal));
        
        // 限制结果数量
        results.truncate(top_k);
        
        Ok(results)
    }
    
    /// 欧氏距离
    fn euclidean_distance(&self, vec1: &[f32], vec2: &[f32]) -> f32 {
        if vec1.len() != vec2.len() {
            return f32::INFINITY;
        }
        
        vec1.iter()
            .zip(vec2.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f32>()
            .sqrt()
    }
    
    /// 使用文本搜索并返回MemoryData
    pub async fn text_to_vector_search(&self, query_text: &str, top_k: usize) -> Result<Vec<MemoryData>> {
        // 将文本转换为向量
        let fake_memory = MemoryData {
            id: uuid::Uuid::new_v4(),
            session_id: uuid::Uuid::new_v4(),
            timestamp: std::time::SystemTime::now(),
            data_type: crate::optimized_persistence::DataType::Knowledge,
            content: serde_json::Value::String(query_text.to_string()),
            metadata: HashMap::new(),
        };
        
        let query_vector = self.encode_to_vector(&fake_memory)?;
        let results = self.similarity_search(query_vector, top_k).await?;
        
        Ok(results.into_iter().map(|r| r.memory).collect())
    }
    
    /// 计算两个向量的相似度
    fn cosine_similarity(&self, vec1: &[f32], vec2: &[f32]) -> f32 {
        if vec1.len() != vec2.len() {
            return 0.0;
        }
        
        let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
        let norm1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm1 * norm2 == 0.0 {
            0.0
        } else {
            dot_product / (norm1 * norm2)
        }
    }
    
    /// 聚类分析
    pub async fn cluster_analysis(&self, min_cluster_size: usize) -> Result<Vec<VectorCluster>> {
        log::info!("执行聚类分析, 最小聚类大小: {}", min_cluster_size);
        
        // 检查缓存
        {
            let clusters = self.clusters.read().await;
            if let Some(ref cached_clusters) = *clusters {
                return Ok(cached_clusters.clone());
            }
        }
        
        let vector_index = self.vector_index.read().await;
        let mut clusters = Vec::new();
        
        if vector_index.len() < min_cluster_size {
            return Ok(clusters);
        }
        
        // 简单的K-means聚类实现
        let k = (vector_index.len() / min_cluster_size).max(2).min(10);
        let mut centroids = self.initialize_centroids(k, &vector_index).await?;
        
        // 迭代优化
        for _iteration in 0..10 {
            let mut assignments: HashMap<usize, Vec<uuid::Uuid>> = HashMap::new();
            
            // 分配点到最近的质心
            for (id, vector) in vector_index.iter() {
                let mut min_distance = f32::INFINITY;
                let mut best_cluster = 0;
                
                for (i, centroid) in centroids.iter().enumerate() {
                    let distance = self.euclidean_distance(vector, centroid);
                    if distance < min_distance {
                        min_distance = distance;
                        best_cluster = i;
                    }
                }
                
                assignments.entry(best_cluster).or_default().push(*id);
            }
            
            // 更新质心
            for (cluster_id, members) in &assignments {
                if !members.is_empty() {
                    let mut new_centroid = vec![0.0; self.dimension];
                    
                    for &member_id in members {
                        if let Some(vector) = vector_index.get(&member_id) {
                            for (i, &value) in vector.iter().enumerate() {
                                new_centroid[i] += value;
                            }
                        }
                    }
                    
                    // 平均化
                    for value in new_centroid.iter_mut() {
                        *value /= members.len() as f32;
                    }
                    
                    centroids[*cluster_id] = new_centroid;
                }
            }
            
            // 创建聚类结果
            clusters.clear();
            for (cluster_id, members) in assignments {
                if members.len() >= min_cluster_size {
                    // 计算半径
                    let centroid = &centroids[cluster_id];
                    let mut max_distance: f32 = 0.0;
                    
                    for &member_id in &members {
                        if let Some(vector) = vector_index.get(&member_id) {
                            let distance = self.euclidean_distance(centroid, vector);
                            max_distance = max_distance.max(distance);
                        }
                    }
                    
                    clusters.push(VectorCluster {
                        id: uuid::Uuid::new_v4(),
                        centroid: centroid.clone(),
                        members,
                        radius: max_distance,
                        created_at: std::time::SystemTime::now(),
                    });
                }
            }
        }
        
        // 缓存结果
        {
            let mut cached_clusters = self.clusters.write().await;
            *cached_clusters = Some(clusters.clone());
        }
        
        Ok(clusters)
    }
    
    async fn initialize_centroids(&self, k: usize, vector_index: &HashMap<uuid::Uuid, Vec<f32>>) -> Result<Vec<Vec<f32>>> {
        let mut centroids = Vec::new();
        let vectors: Vec<_> = vector_index.values().collect();
        
        // 随机选择初始质心
        for i in 0..k {
            let idx = (i * vectors.len()) / k;
            if idx < vectors.len() {
                centroids.push(vectors[idx].clone());
            }
        }
        
        Ok(centroids)
    }
    
    /// 获取向量统计
    pub async fn get_vector_stats(&self) -> Result<VectorStats> {
        let vector_index = self.vector_index.read().await;
        let clusters = self.clusters.read().await;
        
        Ok(VectorStats {
            total_vectors: vector_index.len(),
            dimension: self.dimension,
            clusters_count: clusters.as_ref().map(|c| c.len()).unwrap_or(0),
            avg_cluster_size: if let Some(ref clusters) = *clusters {
                if clusters.is_empty() { 0.0 } else {
                    clusters.iter().map(|c| c.members.len()).sum::<usize>() as f32 / clusters.len() as f32
                }
            } else { 0.0 },
        })
    }
}

/// 向量统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStats {
    pub total_vectors: usize,
    pub dimension: usize,
    pub clusters_count: usize,
    pub avg_cluster_size: f32,
}