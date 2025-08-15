// 语义记忆 - 意义和理解的结晶

use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::{SurrealClient, MemoryData, QueryCondition};
use regex::Regex;
use uuid::Uuid;

/// 语义记忆
pub struct SemanticMemory {
    client: Arc<SurrealClient>,
    // 关键词索引
    keyword_index: Arc<tokio::sync::RwLock<HashMap<String, Vec<uuid::Uuid>>>>,
    // 概念索引  
    concept_index: Arc<tokio::sync::RwLock<HashMap<String, Vec<uuid::Uuid>>>>,
    // 语义网络
    semantic_network: Arc<tokio::sync::RwLock<SemanticNetwork>>,
}

/// 语义记忆条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticEntry {
    pub id: uuid::Uuid,
    pub memory: MemoryData,
    pub keywords: Vec<String>,
    pub concepts: Vec<String>,
    pub sentiment: SentimentScore,
    pub importance: f32,
    pub semantic_embedding: Option<Vec<f32>>,
}

/// 情感分数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentScore {
    pub positive: f32,
    pub negative: f32,
    pub neutral: f32,
    pub confidence: f32,
}

/// 语义网络
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticNetwork {
    pub concept_relations: HashMap<String, Vec<ConceptRelation>>,
    pub keyword_clusters: HashMap<String, Vec<String>>,
}

/// 概念关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptRelation {
    pub from_concept: String,
    pub to_concept: String,
    pub relation_type: String,
    pub strength: f32,
}

impl SemanticMemory {
    pub async fn new(client: Arc<SurrealClient>) -> Result<Self> {
        Ok(Self { 
            client,
            keyword_index: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            concept_index: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            semantic_network: Arc::new(tokio::sync::RwLock::new(SemanticNetwork {
                concept_relations: HashMap::new(),
                keyword_clusters: HashMap::new(),
            })),
        })
    }
    
    /// 存储语义数据
    pub async fn store(&self, memory: MemoryData) -> Result<()> {
        let keywords = self.extract_keywords(&memory);
        let concepts = self.extract_concepts(&memory);
        let sentiment = self.analyze_sentiment(&memory);
        
        // 更新索引
        {
            let mut keyword_index = self.keyword_index.write().await;
            for keyword in &keywords {
                keyword_index.entry(keyword.clone()).or_default().push(memory.id);
            }
        }
        
        {
            let mut concept_index = self.concept_index.write().await;
            for concept in &concepts {
                concept_index.entry(concept.clone()).or_default().push(memory.id);
            }
        }
        
        // 创建语义条目
        let semantic_entry = SemanticEntry {
            id: memory.id,
            memory: memory.clone(),
            keywords,
            concepts,
            sentiment,
            importance: 1.0, // 默认重要性
            semantic_embedding: None, // TODO: 生成语义向量
        };
        
        let semantic_data = serde_json::to_value(&semantic_entry)?;
        self.client.store(memory.id, semantic_data).await
    }
    
    /// 查询语义数据
    pub async fn query(&self, condition: &QueryCondition) -> Result<Vec<MemoryData>> {
        // 使用索引进行快速查询
        let mut candidate_ids: Vec<Uuid> = Vec::new();
        
        if !condition.keywords.is_empty() {
            let keyword_index = self.keyword_index.read().await;
            for keyword in &condition.keywords {
                if let Some(ids) = keyword_index.get(keyword) {
                    candidate_ids.extend(ids.iter().copied());
                }
            }
        }
        
        // 如果没有关键词查询，使用客户端查询
        if candidate_ids.is_empty() {
            return self.client.query_memories(condition).await;
        }
        
        // 去重
        candidate_ids.sort();
        candidate_ids.dedup();
        
        // 从客户端获取完整数据
        let all_memories = self.client.query_memories(condition).await?;
        let mut results = Vec::new();
        
        for memory in all_memories {
            if candidate_ids.contains(&memory.id) {
                results.push(memory);
            }
        }
        
        Ok(results)
    }
    
    /// 提取关键词
    fn extract_keywords(&self, memory: &MemoryData) -> Vec<String> {
        let text = memory.content.to_string();
        let mut keywords = Vec::new();
        
        // 简单的关键词提取逻辑
        let words: Vec<&str> = text.split_whitespace().collect();
        
        for word in words {
            let clean_word = word.to_lowercase()
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_string();
            
            if clean_word.len() > 3 && !self.is_stop_word(&clean_word) {
                keywords.push(clean_word);
            }
        }
        
        // 去重
        keywords.sort();
        keywords.dedup();
        keywords.truncate(10); // 限制数量
        
        keywords
    }
    
    /// 提取概念
    fn extract_concepts(&self, memory: &MemoryData) -> Vec<String> {
        let text = memory.content.to_string();
        let mut concepts = Vec::new();
        
        // 基于模式的概念提取
        let patterns = [
            (r"\b(AI|人工智能|machine learning|机器学习)\b", "AI"),
            (r"\b(浏览器|browser|web|website|网站)\b", "Web"),
            (r"\b(数据|data|database|数据库)\b", "Data"),
            (r"\b(编程|programming|code|代码)\b", "Programming"),
            (r"\b(系统|system|architecture|架构)\b", "System"),
        ];
        
        for (pattern, concept) in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(&text) {
                    concepts.push(concept.to_string());
                }
            }
        }
        
        // 根据数据类型添加概念
        match memory.data_type {
            crate::optimized_persistence::DataType::Perception => concepts.push("Perception".to_string()),
            crate::optimized_persistence::DataType::Action => concepts.push("Action".to_string()),
            crate::optimized_persistence::DataType::Conversation => concepts.push("Communication".to_string()),
            crate::optimized_persistence::DataType::Knowledge => concepts.push("Knowledge".to_string()),
            crate::optimized_persistence::DataType::Experience => concepts.push("Experience".to_string()),
        }
        
        concepts
    }
    
    /// 情感分析
    fn analyze_sentiment(&self, memory: &MemoryData) -> SentimentScore {
        let text = memory.content.to_string().to_lowercase();
        
        // 简单的情感分析
        let positive_words = ["good", "great", "awesome", "excellent", "好", "棒", "优秀"];
        let negative_words = ["bad", "terrible", "awful", "error", "坏", "糟", "错误"];
        
        let mut positive_count = 0;
        let mut negative_count = 0;
        let total_words = text.split_whitespace().count();
        
        for word in positive_words.iter() {
            positive_count += text.matches(word).count();
        }
        
        for word in negative_words.iter() {
            negative_count += text.matches(word).count();
        }
        
        let positive = if total_words > 0 { positive_count as f32 / total_words as f32 } else { 0.0 };
        let negative = if total_words > 0 { negative_count as f32 / total_words as f32 } else { 0.0 };
        let neutral = 1.0 - positive - negative;
        
        SentimentScore {
            positive,
            negative,
            neutral,
            confidence: if positive_count + negative_count > 0 { 0.7 } else { 0.1 },
        }
    }
    
    /// 停用词判断
    fn is_stop_word(&self, word: &str) -> bool {
        let stop_words = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
            "的", "了", "在", "是", "我", "你", "他", "她", "这", "那", "一", "个",
        ];
        stop_words.contains(&word)
    }
    
    /// 语义搜索
    pub async fn semantic_search(&self, query: &str) -> Result<Vec<MemoryData>> {
        log::info!("执行语义搜索: {}", query);
        
        // 提取查询关键词
        let query_keywords = self.extract_keywords_from_text(query);
        
        // 基于关键词的语义搜索
        let condition = QueryCondition {
            session_id: None,
            data_type: None,
            time_range: None,
            keywords: query_keywords.clone(),
            limit: Some(50), // 限制结果数量
        };
        
        let mut results = self.query(&condition).await?;
        
        // 按相似度排序
        results.sort_by(|a, b| {
            let sim_a = self.calculate_similarity(query, &a.content.to_string());
            let sim_b = self.calculate_similarity(query, &b.content.to_string());
            sim_b.partial_cmp(&sim_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        Ok(results)
    }
    
    fn extract_keywords_from_text(&self, text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|w| w.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| w.len() > 2 && !self.is_stop_word(w))
            .collect()
    }
    
    fn calculate_similarity(&self, text1: &str, text2: &str) -> f32 {
        let text1_lower = text1.to_lowercase();
        let text2_lower = text2.to_lowercase();
        let words1: std::collections::HashSet<_> = text1_lower.split_whitespace().collect();
        let words2: std::collections::HashSet<_> = text2_lower.split_whitespace().collect();
        
        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }
    
    /// 查找相似记忆
    pub async fn find_similar(&self, memory_id: uuid::Uuid, threshold: f32) -> Result<Vec<MemoryData>> {
        log::info!("查找相似记忆: {}, 阈值: {}", memory_id, threshold);
        
        // 获取目标记忆
        let target_memory = self.client.query_memories(&QueryCondition {
            session_id: None,
            data_type: None,
            time_range: None,
            keywords: vec![],
            limit: None,
        }).await?;
        
        let target = target_memory.iter().find(|m| m.id == memory_id);
        if let Some(target) = target {
            let target_text = target.content.to_string();
            
            // 获取所有记忆并计算相似度
            let all_memories = self.client.query_memories(&QueryCondition {
                session_id: None,
                data_type: None,
                time_range: None,
                keywords: vec![],
                limit: None,
            }).await?;
            
            let mut similar = Vec::new();
            
            for memory in all_memories {
                if memory.id != memory_id {
                    let similarity = self.calculate_similarity(&target_text, &memory.content.to_string());
                    if similarity >= threshold {
                        similar.push((memory, similarity));
                    }
                }
            }
            
            // 按相似度排序
            similar.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            
            Ok(similar.into_iter().map(|(m, _)| m).collect())
        } else {
            Ok(vec![])
        }
    }
    
    /// 获取语义网络统计
    pub async fn get_semantic_stats(&self) -> Result<SemanticStats> {
        let keyword_index = self.keyword_index.read().await;
        let concept_index = self.concept_index.read().await;
        let network = self.semantic_network.read().await;
        
        Ok(SemanticStats {
            total_keywords: keyword_index.len(),
            total_concepts: concept_index.len(),
            total_relations: network.concept_relations.values().map(|v| v.len()).sum(),
            keyword_clusters: network.keyword_clusters.len(),
            avg_keywords_per_memory: if keyword_index.is_empty() { 0.0 } else {
                keyword_index.values().map(|v| v.len()).sum::<usize>() as f32 / keyword_index.len() as f32
            },
        })
    }
}

/// 语义统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticStats {
    pub total_keywords: usize,
    pub total_concepts: usize,
    pub total_relations: usize,
    pub keyword_clusters: usize,
    pub avg_keywords_per_memory: f32,
}