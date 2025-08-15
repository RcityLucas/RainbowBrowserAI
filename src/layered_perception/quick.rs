// Quick感知层 - <200ms快速感知
// 感官知觉级别，获取基本交互元素和布局

use anyhow::Result;
use super::{QuickData, Element, LayoutInfo, NavPath, ElementType};
use super::lightning::LightningPerception;
use std::time::Instant;

pub struct QuickPerception {
    lightning: LightningPerception,
}

impl QuickPerception {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            lightning: LightningPerception::new().await?,
        })
    }
    
    /// 执行快速感知 - 必须在200ms内完成
    pub async fn perceive(&self, url: &str) -> Result<QuickData> {
        let start = Instant::now();
        
        // 先执行Lightning感知
        let lightning_data = self.lightning.perceive(url).await?;
        
        // 扩展感知：获取所有可交互元素
        let interaction_elements = self.scan_interaction_elements().await?;
        let layout_structure = self.analyze_layout().await?;
        let navigation_paths = self.find_navigation_paths().await?;
        
        let elapsed = start.elapsed().as_millis();
        if elapsed > 200 {
            log::warn!("Quick感知超时: {}ms", elapsed);
        }
        
        Ok(QuickData {
            lightning_data,
            interaction_elements,
            layout_structure,
            navigation_paths,
        })
    }
    
    /// 扫描所有可交互元素
    async fn scan_interaction_elements(&self) -> Result<Vec<Element>> {
        Ok(vec![
            Element {
                selector: "button".to_string(),
                element_type: ElementType::Button,
            },
            Element {
                selector: "a".to_string(),
                element_type: ElementType::Link,
            },
            Element {
                selector: "input".to_string(),
                element_type: ElementType::Input,
            },
            Element {
                selector: "select".to_string(),
                element_type: ElementType::Other,
            },
        ])
    }
    
    /// 分析页面布局
    async fn analyze_layout(&self) -> Result<LayoutInfo> {
        Ok(LayoutInfo {
            structure: "header-main-footer".to_string(),
        })
    }
    
    /// 查找导航路径
    async fn find_navigation_paths(&self) -> Result<Vec<NavPath>> {
        Ok(vec![
            NavPath {
                path: vec!["home".to_string(), "products".to_string()],
            },
        ])
    }
}