// Standard感知层 - <500ms标准感知
// 认知理解级别，完整分析页面内容和结构

use anyhow::Result;
use super::{StandardData, ContentAnalysis, FormInfo, MediaInfo};
use super::quick::QuickPerception;
use std::time::Instant;

pub struct StandardPerception {
    quick: QuickPerception,
}

impl StandardPerception {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            quick: QuickPerception::new().await?,
        })
    }
    
    /// 执行标准感知 - 必须在500ms内完成
    pub async fn perceive(&self, url: &str) -> Result<StandardData> {
        let start = Instant::now();
        
        // 先执行Quick感知
        let quick_data = self.quick.perceive(url).await?;
        
        // 深入分析内容
        let content_analysis = self.analyze_content().await?;
        let form_structures = self.analyze_forms().await?;
        let media_elements = self.scan_media().await?;
        
        let elapsed = start.elapsed().as_millis();
        if elapsed > 500 {
            log::warn!("Standard感知超时: {}ms", elapsed);
        }
        
        Ok(StandardData {
            quick_data,
            content_analysis,
            form_structures,
            media_elements,
        })
    }
    
    /// 分析页面内容
    async fn analyze_content(&self) -> Result<ContentAnalysis> {
        Ok(ContentAnalysis {
            text_content: "页面主要内容...".to_string(),
            keywords: vec![
                "产品".to_string(),
                "服务".to_string(),
                "关于".to_string(),
            ],
        })
    }
    
    /// 分析表单结构
    async fn analyze_forms(&self) -> Result<Vec<FormInfo>> {
        Ok(vec![
            FormInfo {
                form_id: "login-form".to_string(),
                fields: vec![
                    "username".to_string(),
                    "password".to_string(),
                ],
            },
        ])
    }
    
    /// 扫描媒体元素
    async fn scan_media(&self) -> Result<Vec<MediaInfo>> {
        Ok(vec![
            MediaInfo {
                media_type: "image".to_string(),
                url: "/logo.png".to_string(),
            },
        ])
    }
}