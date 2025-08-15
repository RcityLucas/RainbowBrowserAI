//! # 购物助手应用
//! 
//! 智能购物比价和推荐助手

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

/// 购物助手
pub struct ShoppingAssistant {
    executor: SmartExecutor,
    storage: SimpleStorage,
    session_id: String,
    shopping_context: Arc<RwLock<ShoppingContext>>,
    products: Arc<RwLock<Vec<Product>>>,
    cart: Arc<RwLock<ShoppingCart>>,
    price_alerts: Arc<RwLock<Vec<PriceAlert>>>,
    purchase_history: Arc<RwLock<Vec<Purchase>>>,
}

/// 购物上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingContext {
    pub current_search: Option<String>,
    pub preferred_stores: Vec<String>,
    pub budget_limit: Option<f64>,
    pub currency: String,
    pub user_preferences: ShoppingPreferences,
    pub search_filters: ProductFilters,
    pub current_category: Option<ProductCategory>,
}

/// 购物偏好
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingPreferences {
    pub preferred_brands: Vec<String>,
    pub price_range: Option<(f64, f64)>,
    pub quality_level: QualityLevel,
    pub delivery_preference: DeliveryPreference,
    pub payment_methods: Vec<PaymentMethod>,
    pub notification_settings: NotificationSettings,
}

/// 质量级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityLevel {
    Budget,
    Standard,
    Premium,
    Luxury,
}

/// 配送偏好
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryPreference {
    Standard,
    Express,
    SameDay,
    PickupInStore,
}

/// 支付方式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentMethod {
    CreditCard,
    DebitCard,
    PayPal,
    ApplePay,
    GooglePay,
    BankTransfer,
    CashOnDelivery,
}

/// 通知设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub price_drop_alerts: bool,
    pub stock_alerts: bool,
    pub delivery_updates: bool,
    pub promotion_alerts: bool,
}

/// 商品
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: ProductCategory,
    pub brand: String,
    pub price: f64,
    pub currency: String,
    pub discount_price: Option<f64>,
    pub rating: Option<f32>,
    pub review_count: u32,
    pub availability: ProductAvailability,
    pub images: Vec<String>,
    pub specifications: HashMap<String, String>,
    pub seller: Seller,
    pub shipping_info: ShippingInfo,
    pub last_updated: SystemTime,
}

/// 商品类别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProductCategory {
    Electronics,
    Clothing,
    Books,
    Home,
    Beauty,
    Sports,
    Automotive,
    Toys,
    Food,
    Health,
    Other(String),
}

/// 商品可用性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProductAvailability {
    InStock(u32),
    LowStock(u32),
    OutOfStock,
    Discontinued,
    PreOrder,
}

/// 卖家信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Seller {
    pub name: String,
    pub rating: Option<f32>,
    pub reviews_count: u32,
    pub location: String,
    pub verified: bool,
}

/// 物流信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingInfo {
    pub free_shipping: bool,
    pub estimated_days: Option<(u32, u32)>,
    pub shipping_cost: Option<f64>,
    pub express_available: bool,
}

/// 商品过滤器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductFilters {
    pub price_range: Option<(f64, f64)>,
    pub brands: Vec<String>,
    pub categories: Vec<ProductCategory>,
    pub rating_min: Option<f32>,
    pub shipping_free: Option<bool>,
    pub availability_only: bool,
    pub sort_by: SortOption,
}

/// 排序选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOption {
    Relevance,
    PriceLowToHigh,
    PriceHighToLow,
    Rating,
    Newest,
    BestSelling,
}

/// 购物车
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingCart {
    pub id: Uuid,
    pub items: Vec<CartItem>,
    pub total_price: f64,
    pub currency: String,
    pub discount_applied: Option<Discount>,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

/// 购物车项目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItem {
    pub id: Uuid,
    pub product: Product,
    pub quantity: u32,
    pub selected_options: HashMap<String, String>,
    pub added_at: SystemTime,
}

/// 折扣信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discount {
    pub code: String,
    pub description: String,
    pub amount: f64,
    pub discount_type: DiscountType,
}

/// 折扣类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscountType {
    Percentage,
    FixedAmount,
    FreeShipping,
}

/// 价格提醒
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceAlert {
    pub id: Uuid,
    pub product_id: Uuid,
    pub target_price: f64,
    pub current_price: f64,
    pub created_at: SystemTime,
    pub triggered: bool,
    pub triggered_at: Option<SystemTime>,
}

/// 购买记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Purchase {
    pub id: Uuid,
    pub order_number: String,
    pub items: Vec<CartItem>,
    pub total_amount: f64,
    pub currency: String,
    pub purchase_date: SystemTime,
    pub delivery_address: String,
    pub payment_method: PaymentMethod,
    pub status: OrderStatus,
}

/// 订单状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
    Returned,
}

impl ShoppingAssistant {
    /// 创建新的购物助手实例
    pub async fn new() -> Result<Self> {
        let llm_config = LLMConfig::default();
        let executor = SmartExecutor::new(llm_config);
        let storage = SimpleStorage::new();
        let session = storage.create_session();

        Ok(Self {
            executor,
            storage,
            session_id: session.id,
            shopping_context: Arc::new(RwLock::new(ShoppingContext::default())),
            products: Arc::new(RwLock::new(Vec::new())),
            cart: Arc::new(RwLock::new(ShoppingCart::new())),
            price_alerts: Arc::new(RwLock::new(Vec::new())),
            purchase_history: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// 处理购物请求
    pub async fn process_shopping_request(&mut self, user_input: &str) -> Result<ShoppingResponse> {
        // 分析购物意图
        let intent = self.analyze_shopping_intent(user_input).await?;
        
        match intent {
            ShoppingIntent::SearchProduct => self.search_products(user_input).await,
            ShoppingIntent::CompareProducts => self.compare_products(user_input).await,
            ShoppingIntent::AddToCart => self.add_to_cart(user_input).await,
            ShoppingIntent::ManageCart => self.manage_cart(user_input).await,
            ShoppingIntent::CheckPrices => self.check_prices(user_input).await,
            ShoppingIntent::SetPriceAlert => self.set_price_alert(user_input).await,
            ShoppingIntent::ViewHistory => self.view_purchase_history(user_input).await,
            ShoppingIntent::General => self.general_shopping_assistance(user_input).await,
        }
    }

    /// 分析购物意图
    async fn analyze_shopping_intent(&self, user_input: &str) -> Result<ShoppingIntent> {
        let input_lower = user_input.to_lowercase();

        if input_lower.contains("搜索") || input_lower.contains("找") || input_lower.contains("search") {
            Ok(ShoppingIntent::SearchProduct)
        } else if input_lower.contains("比较") || input_lower.contains("对比") || input_lower.contains("compare") {
            Ok(ShoppingIntent::CompareProducts)
        } else if input_lower.contains("加入") || input_lower.contains("购买") || input_lower.contains("add") {
            Ok(ShoppingIntent::AddToCart)
        } else if input_lower.contains("购物车") || input_lower.contains("cart") {
            Ok(ShoppingIntent::ManageCart)
        } else if input_lower.contains("价格") || input_lower.contains("price") {
            Ok(ShoppingIntent::CheckPrices)
        } else if input_lower.contains("提醒") || input_lower.contains("alert") {
            Ok(ShoppingIntent::SetPriceAlert)
        } else if input_lower.contains("历史") || input_lower.contains("history") {
            Ok(ShoppingIntent::ViewHistory)
        } else {
            Ok(ShoppingIntent::General)
        }
    }

    /// 搜索商品
    async fn search_products(&mut self, user_input: &str) -> Result<ShoppingResponse> {
        let search_prompt = format!(
            "用户想要搜索商品：{}。请分析商品需求并提供搜索建议。",
            user_input
        );

        let llm_response = self.executor.execute_smart_request(&search_prompt).await
            .map_err(|e| BrowserError::ExecutionError(e.to_string()))?;

        // 模拟生成搜索结果
        let products = vec![
            Product {
                id: Uuid::new_v4(),
                name: "iPhone 15 Pro".to_string(),
                description: "最新款iPhone，配备A17 Pro芯片".to_string(),
                category: ProductCategory::Electronics,
                brand: "Apple".to_string(),
                price: 7999.0,
                currency: "CNY".to_string(),
                discount_price: Some(7499.0),
                rating: Some(4.5),
                review_count: 1234,
                availability: ProductAvailability::InStock(50),
                images: vec!["iphone15pro_1.jpg".to_string()],
                specifications: {
                    let mut specs = HashMap::new();
                    specs.insert("屏幕".to_string(), "6.1英寸".to_string());
                    specs.insert("存储".to_string(), "256GB".to_string());
                    specs
                },
                seller: Seller {
                    name: "Apple官方店".to_string(),
                    rating: Some(4.8),
                    reviews_count: 50000,
                    location: "中国".to_string(),
                    verified: true,
                },
                shipping_info: ShippingInfo {
                    free_shipping: true,
                    estimated_days: Some((1, 3)),
                    shipping_cost: None,
                    express_available: true,
                },
                last_updated: SystemTime::now(),
            }
        ];

        // 更新上下文
        {
            let mut context = self.shopping_context.write().await;
            context.current_search = Some(user_input.to_string());
        }

        // 保存搜索结果
        {
            let mut product_list = self.products.write().await;
            product_list.extend(products.clone());
        }

        Ok(ShoppingResponse {
            content: llm_response.llm_response,
            products,
            cart: None,
            price_comparisons: Vec::new(),
            suggestions: vec![
                "查看商品详情".to_string(),
                "比较相似商品".to_string(),
                "加入购物车".to_string(),
                "设置价格提醒".to_string(),
            ],
        })
    }

    /// 比较商品
    async fn compare_products(&mut self, user_input: &str) -> Result<ShoppingResponse> {
        let compare_prompt = format!(
            "用户想要比较商品：{}。请提供商品比较分析。",
            user_input
        );

        let llm_response = self.executor.execute_smart_request(&compare_prompt).await
            .map_err(|e| BrowserError::ExecutionError(e.to_string()))?;

        // 生成比较结果
        let comparisons = vec![
            ProductComparison {
                feature: "价格".to_string(),
                product_a_value: "¥7,999".to_string(),
                product_b_value: "¥6,999".to_string(),
                advantage: "产品B更便宜".to_string(),
            },
            ProductComparison {
                feature: "评分".to_string(),
                product_a_value: "4.5/5".to_string(),
                product_b_value: "4.2/5".to_string(),
                advantage: "产品A评分更高".to_string(),
            },
        ];

        Ok(ShoppingResponse {
            content: llm_response.llm_response,
            products: Vec::new(),
            cart: None,
            price_comparisons: comparisons,
            suggestions: vec![
                "查看详细对比".to_string(),
                "选择最适合的产品".to_string(),
                "查看用户评价".to_string(),
            ],
        })
    }

    /// 添加到购物车
    async fn add_to_cart(&mut self, user_input: &str) -> Result<ShoppingResponse> {
        let add_prompt = format!(
            "用户想要添加商品到购物车：{}。请确认操作。",
            user_input
        );

        let llm_response = self.executor.execute_smart_request(&add_prompt).await
            .map_err(|e| BrowserError::ExecutionError(e.to_string()))?;

        // 模拟添加到购物车的逻辑
        let cart = {
            let mut cart = self.cart.write().await;
            cart.total_price += 7999.0;
            cart.updated_at = SystemTime::now();
            cart.clone()
        };

        Ok(ShoppingResponse {
            content: llm_response.llm_response,
            products: Vec::new(),
            cart: Some(cart),
            price_comparisons: Vec::new(),
            suggestions: vec![
                "查看购物车".to_string(),
                "继续购物".to_string(),
                "结算订单".to_string(),
            ],
        })
    }

    /// 管理购物车
    async fn manage_cart(&mut self, _user_input: &str) -> Result<ShoppingResponse> {
        let cart = self.cart.read().await.clone();
        
        let content = format!(
            "您的购物车中有 {} 件商品，总价 ¥{:.2}",
            cart.items.len(),
            cart.total_price
        );

        Ok(ShoppingResponse {
            content,
            products: Vec::new(),
            cart: Some(cart),
            price_comparisons: Vec::new(),
            suggestions: vec![
                "修改商品数量".to_string(),
                "删除商品".to_string(),
                "使用优惠券".to_string(),
                "去结算".to_string(),
            ],
        })
    }

    /// 检查价格
    async fn check_prices(&mut self, user_input: &str) -> Result<ShoppingResponse> {
        let price_prompt = format!(
            "用户想要检查商品价格：{}。请提供价格信息和建议。",
            user_input
        );

        let llm_response = self.executor.execute_smart_request(&price_prompt).await
            .map_err(|e| BrowserError::ExecutionError(e.to_string()))?;

        Ok(ShoppingResponse {
            content: llm_response.llm_response,
            products: Vec::new(),
            cart: None,
            price_comparisons: Vec::new(),
            suggestions: vec![
                "设置价格提醒".to_string(),
                "查看价格历史".to_string(),
                "比较其他商家".to_string(),
            ],
        })
    }

    /// 设置价格提醒
    async fn set_price_alert(&mut self, user_input: &str) -> Result<ShoppingResponse> {
        let alert_prompt = format!(
            "用户想要设置价格提醒：{}。请确认设置。",
            user_input
        );

        let llm_response = self.executor.execute_smart_request(&alert_prompt).await
            .map_err(|e| BrowserError::ExecutionError(e.to_string()))?;

        // 创建价格提醒
        let alert = PriceAlert {
            id: Uuid::new_v4(),
            product_id: Uuid::new_v4(),
            target_price: 6999.0,
            current_price: 7999.0,
            created_at: SystemTime::now(),
            triggered: false,
            triggered_at: None,
        };

        {
            let mut alerts = self.price_alerts.write().await;
            alerts.push(alert);
        }

        Ok(ShoppingResponse {
            content: llm_response.llm_response,
            products: Vec::new(),
            cart: None,
            price_comparisons: Vec::new(),
            suggestions: vec![
                "管理价格提醒".to_string(),
                "查看其他商品".to_string(),
                "继续购物".to_string(),
            ],
        })
    }

    /// 查看购买历史
    async fn view_purchase_history(&mut self, _user_input: &str) -> Result<ShoppingResponse> {
        let history = self.purchase_history.read().await.clone();
        
        let content = if history.is_empty() {
            "您还没有购买记录。".to_string()
        } else {
            format!("您有 {} 条购买记录。", history.len())
        };

        Ok(ShoppingResponse {
            content,
            products: Vec::new(),
            cart: None,
            price_comparisons: Vec::new(),
            suggestions: vec![
                "查看订单详情".to_string(),
                "重新购买".to_string(),
                "评价商品".to_string(),
            ],
        })
    }

    /// 通用购物协助
    async fn general_shopping_assistance(&mut self, user_input: &str) -> Result<ShoppingResponse> {
        let result = self.executor.execute_smart_request(user_input).await
            .map_err(|e| BrowserError::ExecutionError(e.to_string()))?;

        Ok(ShoppingResponse {
            content: result.llm_response,
            products: Vec::new(),
            cart: None,
            price_comparisons: Vec::new(),
            suggestions: vec![
                "搜索商品".to_string(),
                "查看推荐".to_string(),
                "管理购物车".to_string(),
            ],
        })
    }

    /// 获取购物上下文
    pub async fn get_shopping_context(&self) -> ShoppingContext {
        self.shopping_context.read().await.clone()
    }

    /// 获取购物车
    pub async fn get_cart(&self) -> ShoppingCart {
        self.cart.read().await.clone()
    }

    /// 获取价格提醒
    pub async fn get_price_alerts(&self) -> Vec<PriceAlert> {
        self.price_alerts.read().await.clone()
    }
}

/// 购物意图
#[derive(Debug, Clone)]
pub enum ShoppingIntent {
    SearchProduct,
    CompareProducts,
    AddToCart,
    ManageCart,
    CheckPrices,
    SetPriceAlert,
    ViewHistory,
    General,
}

/// 购物助手响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingResponse {
    pub content: String,
    pub products: Vec<Product>,
    pub cart: Option<ShoppingCart>,
    pub price_comparisons: Vec<ProductComparison>,
    pub suggestions: Vec<String>,
}

/// 商品比较
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductComparison {
    pub feature: String,
    pub product_a_value: String,
    pub product_b_value: String,
    pub advantage: String,
}

/// 默认实现
impl Default for ShoppingContext {
    fn default() -> Self {
        Self {
            current_search: None,
            preferred_stores: vec!["淘宝".to_string(), "京东".to_string()],
            budget_limit: None,
            currency: "CNY".to_string(),
            user_preferences: ShoppingPreferences::default(),
            search_filters: ProductFilters::default(),
            current_category: None,
        }
    }
}

impl Default for ShoppingPreferences {
    fn default() -> Self {
        Self {
            preferred_brands: Vec::new(),
            price_range: None,
            quality_level: QualityLevel::Standard,
            delivery_preference: DeliveryPreference::Standard,
            payment_methods: vec![PaymentMethod::CreditCard],
            notification_settings: NotificationSettings {
                price_drop_alerts: true,
                stock_alerts: true,
                delivery_updates: true,
                promotion_alerts: false,
            },
        }
    }
}

impl Default for ProductFilters {
    fn default() -> Self {
        Self {
            price_range: None,
            brands: Vec::new(),
            categories: Vec::new(),
            rating_min: Some(3.0),
            shipping_free: None,
            availability_only: true,
            sort_by: SortOption::Relevance,
        }
    }
}

impl ShoppingCart {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            items: Vec::new(),
            total_price: 0.0,
            currency: "CNY".to_string(),
            discount_applied: None,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }
}