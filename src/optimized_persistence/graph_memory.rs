// 图谱记忆 - 关系和联结的网络

use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::{SurrealClient, MemoryData, QueryCondition};

/// 图谱记忆
pub struct GraphMemory {
    client: Arc<SurrealClient>,
    // 内存中的图结构缓存
    nodes: Arc<tokio::sync::RwLock<HashMap<uuid::Uuid, GraphNode>>>,
    edges: Arc<tokio::sync::RwLock<Vec<GraphEdge>>>,
}

/// 图节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: uuid::Uuid,
    pub data: MemoryData,
    pub connections: Vec<uuid::Uuid>,
    pub importance: f32,
    pub created_at: std::time::SystemTime,
}

/// 图边
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: uuid::Uuid,
    pub from: uuid::Uuid,
    pub to: uuid::Uuid,
    pub relation_type: String,
    pub weight: f32,
    pub created_at: std::time::SystemTime,
}

impl GraphMemory {
    pub async fn new(client: Arc<SurrealClient>) -> Result<Self> {
        Ok(Self { 
            client,
            nodes: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            edges: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        })
    }
    
    /// 存储图谱数据
    pub async fn store(&self, memory: MemoryData) -> Result<()> {
        // 将数据存储为图结构
        let graph_data = serde_json::json!({
            "id": memory.id,
            "type": "graph",
            "nodes": [],
            "edges": [],
            "data": memory,
        });
        
        self.client.store(memory.id, graph_data).await
    }
    
    /// 查询图谱数据
    pub async fn query(&self, condition: &QueryCondition) -> Result<Vec<MemoryData>> {
        // 使用客户端的复杂查询功能
        self.client.query_memories(condition).await
    }
    
    /// 添加关系
    pub async fn add_relation(&self, from: uuid::Uuid, to: uuid::Uuid, relation_type: &str) -> Result<()> {
        log::info!("添加关系: {} -> {} ({})", from, to, relation_type);
        
        let edge = GraphEdge {
            id: uuid::Uuid::new_v4(),
            from,
            to,
            relation_type: relation_type.to_string(),
            weight: 1.0,
            created_at: std::time::SystemTime::now(),
        };
        
        // 添加到内存缓存
        {
            let mut edges = self.edges.write().await;
            edges.push(edge.clone());
        }
        
        // 持久化到数据库
        let edge_data = serde_json::to_value(&edge)?;
        self.client.store(edge.id, edge_data).await?;
        
        Ok(())
    }
    
    /// 查找相关节点
    pub async fn find_related(&self, node_id: uuid::Uuid, depth: usize) -> Result<Vec<uuid::Uuid>> {
        log::info!("查找相关节点: {}, 深度: {}", node_id, depth);
        
        let mut related = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        
        queue.push_back((node_id, 0));
        visited.insert(node_id);
        
        let edges = self.edges.read().await;
        
        while let Some((current, current_depth)) = queue.pop_front() {
            if current_depth >= depth {
                continue;
            }
            
            // 查找直接相关的节点
            for edge in edges.iter() {
                let next_node = if edge.from == current {
                    Some(edge.to)
                } else if edge.to == current {
                    Some(edge.from)
                } else {
                    None
                };
                
                if let Some(next) = next_node {
                    if !visited.contains(&next) {
                        visited.insert(next);
                        related.push(next);
                        queue.push_back((next, current_depth + 1));
                    }
                }
            }
        }
        
        Ok(related)
    }
    
    /// 更新节点重要性
    pub async fn update_importance(&self, node_id: uuid::Uuid, importance: f32) -> Result<()> {
        let mut nodes = self.nodes.write().await;
        if let Some(node) = nodes.get_mut(&node_id) {
            node.importance = importance;
            
            // 持久化更新
            let node_data = serde_json::to_value(&node)?;
            self.client.store(node_id, node_data).await?;
        }
        Ok(())
    }
    
    /// 获取图谱统计
    pub async fn get_graph_stats(&self) -> Result<GraphStats> {
        let nodes = self.nodes.read().await;
        let edges = self.edges.read().await;
        
        Ok(GraphStats {
            node_count: nodes.len(),
            edge_count: edges.len(),
            avg_connections: if nodes.is_empty() { 0.0 } else { edges.len() as f32 / nodes.len() as f32 },
            max_importance: nodes.values().map(|n| n.importance).fold(0.0, f32::max),
        })
    }
}

/// 图谱统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub avg_connections: f32,
    pub max_importance: f32,
}