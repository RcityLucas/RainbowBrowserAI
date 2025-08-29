# Perception Module Development Plan

## Current State Analysis

### What We Have Now (V1 - Basic)
```rust
// Current perception is essentially just DOM queries
pub struct PerceptionEngine {
    mode: PerceptionMode,  // Quick, Standard, Deep
    adaptive: bool,
}
```

**Current Capabilities:**
- ✅ Find elements by CSS selector
- ✅ Extract text content
- ✅ Get page title
- ✅ Basic screenshot capture
- ❌ No semantic understanding
- ❌ No visual recognition
- ❌ No context awareness
- ❌ No intelligent element detection

**Success Rate**: ~20% for real-world pages

## Vision: Intelligent Web Perception System

### Goal
Build a perception module that can understand web pages like a human would - recognizing patterns, understanding context, and intelligently interacting with dynamic content.

## Development Phases

### Phase 1: Foundation (Week 1-2)
**Goal**: Build robust DOM analysis and element detection

#### 1.1 Enhanced DOM Analysis
```rust
pub struct DOMAnalyzer {
    // Complete DOM tree representation
    dom_tree: Node,
    
    // Element categorization
    interactive_elements: Vec<InteractiveElement>,
    form_elements: Vec<FormElement>,
    navigation_elements: Vec<NavElement>,
    content_elements: Vec<ContentElement>,
    
    // Page structure
    layout: PageLayout,
    sections: Vec<PageSection>,
}

pub struct InteractiveElement {
    selector: String,
    element_type: ElementType,
    visible: bool,
    clickable: bool,
    bounds: Rectangle,
    text: String,
    aria_label: Option<String>,
    role: Option<String>,
    confidence: f32,
}
```

#### 1.2 Visual Analysis
```rust
pub struct VisualAnalyzer {
    // Screenshot-based analysis
    screenshot: Vec<u8>,
    
    // Visual elements detection
    buttons: Vec<VisualButton>,
    forms: Vec<VisualForm>,
    images: Vec<VisualImage>,
    
    // Layout understanding
    columns: Vec<Column>,
    rows: Vec<Row>,
    regions: Vec<Region>,
}

impl VisualAnalyzer {
    pub async fn analyze_screenshot(&self) -> VisualAnalysis {
        // Use image processing to detect:
        // - Button-like shapes
        // - Form fields
        // - Text regions
        // - Navigation menus
    }
}
```

#### 1.3 Text Understanding
```rust
pub struct TextAnalyzer {
    // Natural language processing
    page_language: String,
    
    // Text extraction and categorization
    headings: Vec<Heading>,
    paragraphs: Vec<Paragraph>,
    labels: Vec<Label>,
    buttons_text: Vec<ButtonText>,
    
    // Semantic analysis
    keywords: Vec<String>,
    entities: Vec<Entity>,
    intents: Vec<Intent>,
}

impl TextAnalyzer {
    pub fn extract_actionable_text(&self) -> Vec<ActionableText> {
        // Find text that suggests actions:
        // "Click here", "Submit", "Download", "Learn more"
    }
    
    pub fn match_user_intent(&self, command: &str) -> Vec<Element> {
        // Match user's natural language to page elements
        // "sign in" -> finds login button/link
        // "search" -> finds search box
    }
}
```

### Phase 2: Semantic Understanding (Week 3-4)
**Goal**: Understand page meaning and purpose

#### 2.1 Page Type Recognition
```rust
pub enum PageType {
    Homepage,
    LoginPage,
    SearchResults,
    ProductPage,
    ArticlePage,
    FormPage,
    Dashboard,
    Checkout,
    Unknown,
}

pub struct PageClassifier {
    pub fn classify(&self, dom: &DOM, visual: &VisualAnalysis) -> PageType {
        // Use ML or heuristics to classify page type
        // This helps predict what elements to look for
    }
}
```

#### 2.2 Element Role Detection
```rust
pub struct ElementRoleDetector {
    pub fn detect_role(&self, element: &Element) -> ElementRole {
        // Combine multiple signals:
        // - HTML tag and attributes
        // - ARIA roles
        // - Visual appearance
        // - Surrounding context
        // - Text content
        
        ElementRole::Button {
            action: ButtonAction::Submit,
            confidence: 0.95,
        }
    }
}

pub enum ElementRole {
    Button { action: ButtonAction },
    Input { input_type: InputType },
    Link { link_type: LinkType },
    Navigation { nav_type: NavType },
    Content { content_type: ContentType },
}
```

#### 2.3 Form Understanding
```rust
pub struct FormAnalyzer {
    pub fn analyze_form(&self, form: &Element) -> FormStructure {
        FormStructure {
            fields: vec![
                FormField {
                    label: "Email Address",
                    input_selector: "input#email",
                    field_type: FieldType::Email,
                    required: true,
                    validation: Some("email"),
                },
                // ... more fields
            ],
            submit_button: "button[type=submit]",
            action: "/api/login",
            method: "POST",
        }
    }
    
    pub fn fill_form(&self, data: HashMap<String, String>) -> FillPlan {
        // Generate plan to fill form with provided data
        // Match data keys to form fields intelligently
    }
}
```

### Phase 3: Context-Aware Interaction (Week 5-6)
**Goal**: Maintain context and handle dynamic content

#### 3.1 Context Management
```rust
pub struct PerceptionContext {
    // Page history
    current_page: PageSnapshot,
    previous_pages: Vec<PageSnapshot>,
    
    // Interaction history
    last_clicked: Option<Element>,
    last_typed: Option<String>,
    form_state: HashMap<String, String>,
    
    // Element memory
    named_elements: HashMap<String, Element>,  // "the red button" -> element
    focused_element: Option<Element>,
    
    // User preferences learned
    interaction_patterns: Vec<Pattern>,
}

impl PerceptionContext {
    pub fn resolve_reference(&self, reference: &str) -> Option<Element> {
        // Resolve "it", "that button", "the form", etc.
        match reference {
            "it" | "that" => self.last_interacted_element(),
            "the form" => self.current_form(),
            _ => self.named_elements.get(reference),
        }
    }
}
```

#### 3.2 Dynamic Content Handling
```rust
pub struct DynamicContentHandler {
    pub async fn wait_for_element(&self, description: &str) -> Result<Element> {
        // Intelligent waiting with multiple strategies
        let strategies = vec![
            WaitStrategy::Visible,
            WaitStrategy::Clickable,
            WaitStrategy::TextContent,
            WaitStrategy::NetworkIdle,
        ];
        
        // Try strategies until element appears
    }
    
    pub async fn handle_popup(&self) -> Result<()> {
        // Detect and handle popups, modals, alerts
    }
    
    pub async fn handle_infinite_scroll(&self) -> Result<Vec<Element>> {
        // Scroll and load more content intelligently
    }
}
```

### Phase 4: Intelligent Element Selection (Week 7-8)
**Goal**: Find elements using natural language

#### 4.1 Natural Language Element Finder
```rust
pub struct NaturalLanguageSelector {
    pub fn find_element(&self, description: &str) -> Result<Element> {
        // "the big blue button" -> visual + text analysis
        // "login button" -> semantic understanding
        // "price under $50" -> content analysis
        // "third link in menu" -> structural understanding
        
        let candidates = self.find_candidates(description);
        let scored = self.score_candidates(candidates, description);
        scored.into_iter().max_by_key(|c| c.score)
    }
    
    fn score_candidates(&self, candidates: Vec<Element>, description: &str) -> Vec<ScoredElement> {
        candidates.into_iter().map(|elem| {
            let text_score = self.text_similarity(&elem.text, description);
            let visual_score = self.visual_match(&elem, description);
            let context_score = self.context_match(&elem, description);
            let position_score = self.position_match(&elem, description);
            
            ScoredElement {
                element: elem,
                score: text_score * 0.4 + visual_score * 0.3 
                       + context_score * 0.2 + position_score * 0.1,
            }
        }).collect()
    }
}
```

#### 4.2 Fuzzy Matching
```rust
pub struct FuzzyMatcher {
    pub fn match_text(&self, target: &str, candidates: Vec<String>) -> Vec<Match> {
        // Handle typos, synonyms, abbreviations
        // "Sign in" matches "Login", "Log in", "Sign-in", etc.
    }
    
    pub fn match_visual(&self, description: &str, element: &VisualElement) -> f32 {
        // "red button" -> check color
        // "large text" -> check font size
        // "top right" -> check position
    }
}
```

### Phase 5: Advanced Features (Week 9-10)
**Goal**: Handle complex scenarios

#### 5.1 Multi-Frame/iframe Support
```rust
pub struct FrameManager {
    main_frame: Frame,
    iframes: Vec<Frame>,
    
    pub fn find_element_across_frames(&self, selector: &str) -> Option<(Frame, Element)> {
        // Search across all frames
    }
}
```

#### 5.2 Shadow DOM Support
```rust
pub struct ShadowDOMHandler {
    pub fn pierce_shadow_dom(&self, host: &Element) -> Vec<Element> {
        // Access elements inside shadow roots
    }
}
```

#### 5.3 Accessibility Analysis
```rust
pub struct AccessibilityAnalyzer {
    pub fn analyze(&self) -> AccessibilityReport {
        AccessibilityReport {
            keyboard_navigable: true,
            screen_reader_friendly: true,
            color_contrast_ok: false,
            missing_alt_text: vec![...],
            aria_issues: vec![...],
        }
    }
}
```

## Implementation Roadmap

### Week 1-2: Foundation
- [ ] Implement enhanced DOM analyzer
- [ ] Build element categorization system
- [ ] Create visual analysis module
- [ ] Add text extraction and analysis

### Week 3-4: Semantic Understanding  
- [ ] Implement page type classifier
- [ ] Build element role detector
- [ ] Create form analyzer
- [ ] Add semantic matching

### Week 5-6: Context Management
- [ ] Build context system
- [ ] Implement element memory
- [ ] Add reference resolution
- [ ] Create dynamic content handlers

### Week 7-8: Natural Language
- [ ] Build natural language selector
- [ ] Implement fuzzy matching
- [ ] Add scoring system
- [ ] Create candidate ranking

### Week 9-10: Advanced Features
- [ ] Add iframe support
- [ ] Handle shadow DOM
- [ ] Build accessibility analyzer
- [ ] Create perception testing framework

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_find_button_by_text() {
    let html = r#"<button>Click me</button>"#;
    let perception = PerceptionEngine::new();
    let element = perception.find_element("click button").unwrap();
    assert_eq!(element.tag_name, "button");
}
```

### Integration Tests
```rust
#[test]
async fn test_real_website_perception() {
    let perception = PerceptionEngine::new();
    perception.navigate("https://example.com").await;
    
    // Should understand common elements
    assert!(perception.find_element("search box").is_ok());
    assert!(perception.find_element("login button").is_ok());
    assert!(perception.find_element("main navigation").is_ok());
}
```

### Benchmark Suite
- Test on top 100 websites
- Measure element finding accuracy
- Track performance metrics
- Compare with human testers

## Success Metrics

### Phase 1 Target (Foundation)
- Element detection accuracy: 70%
- DOM analysis time: <500ms
- Visual analysis time: <1s

### Phase 2 Target (Semantic)
- Page type classification: 85% accuracy
- Form field detection: 90% accuracy
- Button purpose detection: 80% accuracy

### Phase 3 Target (Context)
- Reference resolution: 75% accuracy
- Dynamic content handling: 80% success
- Context retention: 95% accuracy

### Phase 4 Target (Natural Language)
- Natural language element finding: 70% success
- Fuzzy matching accuracy: 85%
- Multi-criteria scoring: 80% relevance

### Phase 5 Target (Advanced)
- iframe support: 95% coverage
- Shadow DOM: 90% coverage
- Overall perception accuracy: 85%

## Code Structure

```
poc/src/perception/
├── mod.rs                 # Main perception module
├── dom/
│   ├── analyzer.rs       # DOM analysis
│   ├── categorizer.rs    # Element categorization
│   └── extractor.rs      # Data extraction
├── visual/
│   ├── analyzer.rs       # Visual analysis
│   ├── detector.rs       # Visual element detection
│   └── processor.rs      # Image processing
├── semantic/
│   ├── classifier.rs     # Page classification
│   ├── role_detector.rs  # Element roles
│   └── form_analyzer.rs  # Form understanding
├── context/
│   ├── manager.rs        # Context management
│   ├── memory.rs         # Element memory
│   └── resolver.rs       # Reference resolution
├── nlp/
│   ├── selector.rs       # Natural language selection
│   ├── fuzzy.rs          # Fuzzy matching
│   └── scorer.rs         # Scoring system
└── tests/
    ├── unit/             # Unit tests
    ├── integration/      # Integration tests
    └── benchmarks/       # Performance tests
```

## Next Steps

1. **Start with Phase 1**: Focus on robust DOM and visual analysis
2. **Build incrementally**: Each phase builds on the previous
3. **Test continuously**: Validate on real websites
4. **Gather feedback**: Test with actual users
5. **Iterate and improve**: Refine based on results

This plan will transform the basic perception module into an intelligent system that can truly understand and interact with web pages like a human would.