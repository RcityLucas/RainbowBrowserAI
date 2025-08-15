//! # 旅行助手应用
//! 
//! 专业的旅行规划和预订助手

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::time::SystemTime;

use crate::core::{llm::*, executor::*};
use crate::base::storage::*;
use crate::types::*;
use crate::error::*;

/// 旅行助手
pub struct TravelAssistant {
    executor: SmartExecutor,
    storage: SimpleStorage,
    session_id: String,
    travel_context: Arc<RwLock<TravelContext>>,
    destinations: Arc<RwLock<Vec<Destination>>>,
    bookings: Arc<RwLock<Vec<TravelBooking>>>,
}

/// 旅行上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TravelContext {
    pub current_trip: Option<TripPlan>,
    pub user_preferences: TravelPreferences,
    pub search_filters: SearchFilters,
    pub budget: Option<Budget>,
    pub travel_dates: Option<DateRange>,
    pub travelers_count: u32,
}

/// 旅行计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TripPlan {
    pub id: Uuid,
    pub title: String,
    pub destinations: Vec<Destination>,
    pub duration_days: u32,
    pub estimated_cost: Option<f64>,
    pub itinerary: Vec<ItineraryItem>,
    pub created_at: SystemTime,
    pub status: TripStatus,
}

/// 目的地
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Destination {
    pub id: Uuid,
    pub name: String,
    pub country: String,
    pub city: Option<String>,
    pub coordinates: Option<(f64, f64)>,
    pub description: Option<String>,
    pub best_time_to_visit: Option<String>,
    pub estimated_days: u32,
    pub attractions: Vec<Attraction>,
}

/// 景点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attraction {
    pub name: String,
    pub description: String,
    pub category: AttractionCategory,
    pub rating: Option<f32>,
    pub estimated_time: Option<String>,
    pub cost: Option<f64>,
}

/// 景点类别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttractionCategory {
    Historical,
    Natural,
    Cultural,
    Entertainment,
    Shopping,
    Religious,
    Adventure,
    Food,
}

/// 行程项目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItineraryItem {
    pub id: Uuid,
    pub day: u32,
    pub start_time: String,
    pub end_time: String,
    pub activity: String,
    pub location: String,
    pub description: Option<String>,
    pub cost: Option<f64>,
    pub booking_required: bool,
}

/// 旅行状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TripStatus {
    Planning,
    Confirmed,
    InProgress,
    Completed,
    Cancelled,
}

/// 旅行偏好
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TravelPreferences {
    pub accommodation_type: Vec<AccommodationType>,
    pub transportation_type: Vec<TransportationType>,
    pub food_preferences: Vec<String>,
    pub activity_types: Vec<ActivityType>,
    pub budget_level: BudgetLevel,
    pub travel_style: TravelStyle,
}

/// 住宿类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccommodationType {
    Hotel,
    Hostel,
    Apartment,
    Resort,
    BedAndBreakfast,
    Camping,
}

/// 交通方式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportationType {
    Flight,
    Train,
    Bus,
    Car,
    Bike,
    Walking,
}

/// 活动类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityType {
    Sightseeing,
    Adventure,
    Relaxation,
    Cultural,
    Food,
    Shopping,
    Nature,
    Photography,
}

/// 预算级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BudgetLevel {
    Budget,
    Mid,
    Luxury,
    Unlimited,
}

/// 旅行风格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TravelStyle {
    Solo,
    Couple,
    Family,
    Group,
    Business,
    Adventure,
    Luxury,
    Backpacking,
}

/// 搜索过滤器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub price_range: Option<(f64, f64)>,
    pub rating_min: Option<f32>,
    pub duration_range: Option<(u32, u32)>,
    pub activity_types: Vec<ActivityType>,
    pub accommodation_types: Vec<AccommodationType>,
}

/// 预算
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    pub total_budget: f64,
    pub currency: String,
    pub breakdown: HashMap<String, f64>, // 分类预算
}

/// 日期范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start_date: SystemTime,
    pub end_date: SystemTime,
}

/// 旅行预订
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TravelBooking {
    pub id: Uuid,
    pub booking_type: BookingType,
    pub provider: String,
    pub reference_number: String,
    pub cost: f64,
    pub currency: String,
    pub status: BookingStatus,
    pub created_at: SystemTime,
    pub details: BookingDetails,
}

/// 预订类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BookingType {
    Flight,
    Hotel,
    CarRental,
    Activity,
    Restaurant,
    Train,
    Bus,
}

/// 预订状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BookingStatus {
    Pending,
    Confirmed,
    Cancelled,
    Completed,
}

/// 预订详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookingDetails {
    pub description: String,
    pub dates: DateRange,
    pub location: String,
    pub participants: u32,
    pub special_requests: Vec<String>,
}

impl TravelAssistant {
    /// 创建新的旅行助手实例
    pub async fn new() -> Result<Self> {
        let llm_config = LLMConfig::default();
        let executor = SmartExecutor::new(llm_config);
        let storage = SimpleStorage::new();
        let session = storage.create_session();

        Ok(Self {
            executor,
            storage,
            session_id: session.id,
            travel_context: Arc::new(RwLock::new(TravelContext::default())),
            destinations: Arc::new(RwLock::new(Vec::new())),
            bookings: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// 处理旅行请求
    pub async fn process_travel_request(&mut self, user_input: &str) -> Result<TravelResponse> {
        // 分析旅行意图
        let intent = self.analyze_travel_intent(user_input).await?;
        
        match intent {
            TravelIntent::PlanTrip => self.plan_trip(user_input).await,
            TravelIntent::SearchDestinations => self.search_destinations(user_input).await,
            TravelIntent::BookAccommodation => self.book_accommodation(user_input).await,
            TravelIntent::BookTransportation => self.book_transportation(user_input).await,
            TravelIntent::FindActivities => self.find_activities(user_input).await,
            TravelIntent::ManageBookings => self.manage_bookings(user_input).await,
            TravelIntent::General => self.general_travel_assistance(user_input).await,
        }
    }

    /// 分析旅行意图
    async fn analyze_travel_intent(&self, user_input: &str) -> Result<TravelIntent> {
        let input_lower = user_input.to_lowercase();

        if input_lower.contains("规划") || input_lower.contains("计划") || input_lower.contains("plan") {
            Ok(TravelIntent::PlanTrip)
        } else if input_lower.contains("搜索") || input_lower.contains("推荐") || input_lower.contains("目的地") {
            Ok(TravelIntent::SearchDestinations)
        } else if input_lower.contains("酒店") || input_lower.contains("住宿") || input_lower.contains("hotel") {
            Ok(TravelIntent::BookAccommodation)
        } else if input_lower.contains("机票") || input_lower.contains("交通") || input_lower.contains("flight") {
            Ok(TravelIntent::BookTransportation)
        } else if input_lower.contains("活动") || input_lower.contains("景点") || input_lower.contains("activity") {
            Ok(TravelIntent::FindActivities)
        } else if input_lower.contains("预订") || input_lower.contains("booking") {
            Ok(TravelIntent::ManageBookings)
        } else {
            Ok(TravelIntent::General)
        }
    }

    /// 规划旅行
    async fn plan_trip(&mut self, user_input: &str) -> Result<TravelResponse> {
        // 使用LLM分析用户需求
        let plan_prompt = format!(
            "用户想要规划旅行：{}。请提供旅行建议，包括目的地、行程、预算等。",
            user_input
        );
        
        let llm_response = self.executor.execute_smart_request(&plan_prompt).await
            .map_err(|e| BrowserError::ExecutionError(e.to_string()))?;

        // 创建新的旅行计划
        let trip_plan = TripPlan {
            id: Uuid::new_v4(),
            title: "新旅行计划".to_string(),
            destinations: Vec::new(),
            duration_days: 7, // 默认7天
            estimated_cost: None,
            itinerary: Vec::new(),
            created_at: SystemTime::now(),
            status: TripStatus::Planning,
        };

        // 更新上下文
        {
            let mut context = self.travel_context.write().await;
            context.current_trip = Some(trip_plan.clone());
        }

        Ok(TravelResponse {
            content: llm_response.llm_response,
            trip_plan: Some(trip_plan),
            destinations: Vec::new(),
            bookings: Vec::new(),
            suggestions: vec![
                "设置预算范围".to_string(),
                "选择旅行日期".to_string(),
                "确定旅行风格".to_string(),
            ],
        })
    }

    /// 搜索目的地
    async fn search_destinations(&mut self, user_input: &str) -> Result<TravelResponse> {
        let search_prompt = format!(
            "用户想要搜索旅行目的地：{}。请推荐合适的目的地。",
            user_input
        );

        let llm_response = self.executor.execute_smart_request(&search_prompt).await
            .map_err(|e| BrowserError::ExecutionError(e.to_string()))?;

        // 生成示例目的地
        let destinations = vec![
            Destination {
                id: Uuid::new_v4(),
                name: "京都".to_string(),
                country: "日本".to_string(),
                city: Some("京都市".to_string()),
                coordinates: Some((35.0116, 135.7681)),
                description: Some("历史悠久的古都，拥有众多寺庙和传统文化".to_string()),
                best_time_to_visit: Some("春季和秋季".to_string()),
                estimated_days: 3,
                attractions: vec![
                    Attraction {
                        name: "清水寺".to_string(),
                        description: "著名的佛教寺庙".to_string(),
                        category: AttractionCategory::Religious,
                        rating: Some(4.5),
                        estimated_time: Some("2-3小时".to_string()),
                        cost: Some(400.0),
                    }
                ],
            }
        ];

        {
            let mut dest_list = self.destinations.write().await;
            dest_list.extend(destinations.clone());
        }

        Ok(TravelResponse {
            content: llm_response.llm_response,
            trip_plan: None,
            destinations,
            bookings: Vec::new(),
            suggestions: vec![
                "查看详细信息".to_string(),
                "添加到行程".to_string(),
                "比较其他目的地".to_string(),
            ],
        })
    }

    /// 预订住宿
    async fn book_accommodation(&mut self, user_input: &str) -> Result<TravelResponse> {
        let booking_prompt = format!(
            "用户想要预订住宿：{}。请提供住宿建议和预订信息。",
            user_input
        );

        let llm_response = self.executor.execute_smart_request(&booking_prompt).await
            .map_err(|e| BrowserError::ExecutionError(e.to_string()))?;

        Ok(TravelResponse {
            content: llm_response.llm_response,
            trip_plan: None,
            destinations: Vec::new(),
            bookings: Vec::new(),
            suggestions: vec![
                "比较价格".to_string(),
                "查看评价".to_string(),
                "确认预订".to_string(),
            ],
        })
    }

    /// 预订交通
    async fn book_transportation(&mut self, user_input: &str) -> Result<TravelResponse> {
        let transport_prompt = format!(
            "用户想要预订交通：{}。请提供交通建议和预订信息。",
            user_input
        );

        let llm_response = self.executor.execute_smart_request(&transport_prompt).await
            .map_err(|e| BrowserError::ExecutionError(e.to_string()))?;

        Ok(TravelResponse {
            content: llm_response.llm_response,
            trip_plan: None,
            destinations: Vec::new(),
            bookings: Vec::new(),
            suggestions: vec![
                "比较航班".to_string(),
                "选择座位".to_string(),
                "确认预订".to_string(),
            ],
        })
    }

    /// 查找活动
    async fn find_activities(&mut self, user_input: &str) -> Result<TravelResponse> {
        let activity_prompt = format!(
            "用户想要查找旅行活动：{}。请推荐合适的活动。",
            user_input
        );

        let llm_response = self.executor.execute_smart_request(&activity_prompt).await
            .map_err(|e| BrowserError::ExecutionError(e.to_string()))?;

        Ok(TravelResponse {
            content: llm_response.llm_response,
            trip_plan: None,
            destinations: Vec::new(),
            bookings: Vec::new(),
            suggestions: vec![
                "查看活动详情".to_string(),
                "检查可用性".to_string(),
                "立即预订".to_string(),
            ],
        })
    }

    /// 管理预订
    async fn manage_bookings(&mut self, _user_input: &str) -> Result<TravelResponse> {
        let bookings = self.bookings.read().await.clone();
        
        let content = if bookings.is_empty() {
            "您目前没有任何预订记录。".to_string()
        } else {
            format!("您有 {} 个预订记录。", bookings.len())
        };

        Ok(TravelResponse {
            content,
            trip_plan: None,
            destinations: Vec::new(),
            bookings,
            suggestions: vec![
                "查看预订详情".to_string(),
                "修改预订".to_string(),
                "取消预订".to_string(),
            ],
        })
    }

    /// 通用旅行协助
    async fn general_travel_assistance(&mut self, user_input: &str) -> Result<TravelResponse> {
        let result = self.executor.execute_smart_request(user_input).await
            .map_err(|e| BrowserError::ExecutionError(e.to_string()))?;

        Ok(TravelResponse {
            content: result.llm_response,
            trip_plan: None,
            destinations: Vec::new(),
            bookings: Vec::new(),
            suggestions: vec![
                "开始规划新旅行".to_string(),
                "搜索目的地".to_string(),
                "管理现有预订".to_string(),
            ],
        })
    }

    /// 获取旅行上下文
    pub async fn get_travel_context(&self) -> TravelContext {
        self.travel_context.read().await.clone()
    }

    /// 获取所有目的地
    pub async fn get_destinations(&self) -> Vec<Destination> {
        self.destinations.read().await.clone()
    }

    /// 获取所有预订
    pub async fn get_bookings(&self) -> Vec<TravelBooking> {
        self.bookings.read().await.clone()
    }
}

/// 旅行意图
#[derive(Debug, Clone)]
pub enum TravelIntent {
    PlanTrip,
    SearchDestinations,
    BookAccommodation,
    BookTransportation,
    FindActivities,
    ManageBookings,
    General,
}

/// 旅行助手响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TravelResponse {
    pub content: String,
    pub trip_plan: Option<TripPlan>,
    pub destinations: Vec<Destination>,
    pub bookings: Vec<TravelBooking>,
    pub suggestions: Vec<String>,
}

/// 默认实现
impl Default for TravelContext {
    fn default() -> Self {
        Self {
            current_trip: None,
            user_preferences: TravelPreferences::default(),
            search_filters: SearchFilters::default(),
            budget: None,
            travel_dates: None,
            travelers_count: 1,
        }
    }
}

impl Default for TravelPreferences {
    fn default() -> Self {
        Self {
            accommodation_type: vec![AccommodationType::Hotel],
            transportation_type: vec![TransportationType::Flight],
            food_preferences: vec!["当地特色".to_string()],
            activity_types: vec![ActivityType::Sightseeing],
            budget_level: BudgetLevel::Mid,
            travel_style: TravelStyle::Solo,
        }
    }
}

impl Default for SearchFilters {
    fn default() -> Self {
        Self {
            price_range: None,
            rating_min: Some(3.0),
            duration_range: None,
            activity_types: Vec::new(),
            accommodation_types: Vec::new(),
        }
    }
}