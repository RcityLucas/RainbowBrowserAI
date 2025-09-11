use super::traits::{Tool, ToolCategory};
use crate::browser::Browser;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::info;

// ============================================================================
// Create Test Fixture Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTestFixtureInput {
    pub fixture_type: TestFixtureType,
    #[serde(default)]
    pub options: TestFixtureOptions,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestFixtureType {
    SimpleForm,
    ComplexForm,
    DataTable,
    NavigationMenu,
    ContentPage,
    LoadingStates,
    ErrorPage,
    Interactive,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TestFixtureOptions {
    #[serde(default)]
    pub include_validation: bool,
    #[serde(default)]
    pub add_animations: bool,
    #[serde(default)]
    pub responsive_design: bool,
    #[serde(default)]
    pub accessibility_features: bool,
    #[serde(default = "default_element_count")]
    pub element_count: usize,
}

fn default_element_count() -> usize {
    10
}

#[derive(Debug, Serialize)]
pub struct CreateTestFixtureOutput {
    pub success: bool,
    pub fixture_type: String,
    pub url: String,
    pub elements_created: usize,
    pub test_selectors: Vec<String>,
    pub fixture_description: String,
}

pub struct CreateTestFixtureTool {
    browser: Arc<Browser>,
}

impl CreateTestFixtureTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }

    async fn create_simple_form(&self, options: &TestFixtureOptions) -> Result<CreateTestFixtureOutput> {
        let html = self.generate_simple_form_html(options);
        self.inject_test_html(&html).await?;

        Ok(CreateTestFixtureOutput {
            success: true,
            fixture_type: "simple_form".to_string(),
            url: "data:text/html".to_string(),
            elements_created: 6,
            test_selectors: vec![
                "#test-name".to_string(),
                "#test-email".to_string(),
                "#test-submit".to_string(),
                "#test-form".to_string(),
                ".form-group".to_string(),
                "input[type='text']".to_string(),
            ],
            fixture_description: "Simple form with name, email, and submit button".to_string(),
        })
    }

    async fn create_complex_form(&self, options: &TestFixtureOptions) -> Result<CreateTestFixtureOutput> {
        let html = self.generate_complex_form_html(options);
        self.inject_test_html(&html).await?;

        Ok(CreateTestFixtureOutput {
            success: true,
            fixture_type: "complex_form".to_string(),
            url: "data:text/html".to_string(),
            elements_created: 15,
            test_selectors: vec![
                "#test-form".to_string(),
                "#personal-info".to_string(),
                "#contact-info".to_string(),
                "#preferences".to_string(),
                "select#country".to_string(),
                "input[type='radio']".to_string(),
                "input[type='checkbox']".to_string(),
                "textarea#comments".to_string(),
            ],
            fixture_description: "Complex multi-section form with various input types".to_string(),
        })
    }

    async fn create_data_table(&self, options: &TestFixtureOptions) -> Result<CreateTestFixtureOutput> {
        let html = self.generate_data_table_html(options);
        self.inject_test_html(&html).await?;

        Ok(CreateTestFixtureOutput {
            success: true,
            fixture_type: "data_table".to_string(),
            url: "data:text/html".to_string(),
            elements_created: options.element_count * 4, // rows * columns
            test_selectors: vec![
                "#test-table".to_string(),
                "thead".to_string(),
                "tbody".to_string(),
                "th".to_string(),
                "td".to_string(),
                "tr:nth-child(odd)".to_string(),
                "tr:nth-child(even)".to_string(),
                ".sortable".to_string(),
            ],
            fixture_description: format!("Data table with {} rows of sample data", options.element_count),
        })
    }

    async fn create_navigation_menu(&self, options: &TestFixtureOptions) -> Result<CreateTestFixtureOutput> {
        let html = self.generate_navigation_html(options);
        self.inject_test_html(&html).await?;

        Ok(CreateTestFixtureOutput {
            success: true,
            fixture_type: "navigation_menu".to_string(),
            url: "data:text/html".to_string(),
            elements_created: options.element_count + 3,
            test_selectors: vec![
                "#main-nav".to_string(),
                ".nav-item".to_string(),
                ".dropdown".to_string(),
                ".dropdown-menu".to_string(),
                "a[href]".to_string(),
                ".active".to_string(),
                ".nav-brand".to_string(),
            ],
            fixture_description: format!("Navigation menu with {} items and dropdowns", options.element_count),
        })
    }

    async fn inject_test_html(&self, html: &str) -> Result<()> {
        // Use JavaScript to inject HTML directly into the page
        let script = format!(
            "document.documentElement.innerHTML = {};",
            serde_json::to_string(html)?
        );
        
        // First navigate to a blank page
        self.browser.navigate_to("about:blank").await?;
        
        // Then inject the test HTML
        self.browser.execute_script(&script).await?;
        
        // Wait for page to load
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        Ok(())
    }

    fn generate_simple_form_html(&self, options: &TestFixtureOptions) -> String {
        let accessibility_attrs = if options.accessibility_features {
            r#"role="form" aria-labelledby="form-title""#
        } else {
            ""
        };

        let validation_script = if options.include_validation {
            r#"
            <script>
                document.getElementById('test-form').addEventListener('submit', function(e) {
                    const name = document.getElementById('test-name').value;
                    const email = document.getElementById('test-email').value;
                    
                    if (!name || !email || !email.includes('@')) {
                        e.preventDefault();
                        alert('Please fill in all fields with valid data');
                    }
                });
            </script>
            "#
        } else {
            ""
        };

        format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Test Form Fixture</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }}
        .form-container {{ max-width: 500px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
        .form-group {{ margin-bottom: 20px; }}
        label {{ display: block; margin-bottom: 5px; font-weight: bold; color: #333; }}
        input[type="text"], input[type="email"] {{ width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; font-size: 16px; }}
        input:focus {{ outline: none; border-color: #4CAF50; box-shadow: 0 0 5px rgba(76,175,80,0.3); }}
        button {{ background: #4CAF50; color: white; padding: 12px 24px; border: none; border-radius: 4px; cursor: pointer; font-size: 16px; }}
        button:hover {{ background: #45a049; }}
        .test-indicator {{ position: fixed; top: 10px; right: 10px; background: #ff6b35; color: white; padding: 8px 16px; border-radius: 4px; font-weight: bold; }}
    </style>
</head>
<body>
    <div class="test-indicator">TEST FIXTURE</div>
    <div class="form-container">
        <h1 id="form-title">Contact Information</h1>
        <form id="test-form" {accessibility_attrs}>
            <div class="form-group">
                <label for="test-name">Full Name:</label>
                <input type="text" id="test-name" name="name" placeholder="Enter your full name" required>
            </div>
            
            <div class="form-group">
                <label for="test-email">Email Address:</label>
                <input type="email" id="test-email" name="email" placeholder="your.email@example.com" required>
            </div>
            
            <div class="form-group">
                <button type="submit" id="test-submit">Submit Form</button>
            </div>
        </form>
    </div>
    {validation_script}
</body>
</html>
        "#, accessibility_attrs=accessibility_attrs, validation_script=validation_script)
    }

    fn generate_complex_form_html(&self, _options: &TestFixtureOptions) -> String {
        format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Complex Form Test Fixture</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; background: #f8f9fa; }}
        .form-container {{ max-width: 800px; margin: 0 auto; background: white; padding: 40px; border-radius: 12px; box-shadow: 0 4px 20px rgba(0,0,0,0.1); }}
        .section {{ margin-bottom: 30px; padding: 20px; border: 1px solid #e9ecef; border-radius: 8px; }}
        .section h3 {{ margin-top: 0; color: #495057; border-bottom: 2px solid #007bff; padding-bottom: 10px; }}
        .form-row {{ display: flex; gap: 20px; margin-bottom: 15px; }}
        .form-group {{ flex: 1; }}
        label {{ display: block; margin-bottom: 5px; font-weight: 600; color: #333; }}
        input, select, textarea {{ width: 100%; padding: 12px; border: 1px solid #ced4da; border-radius: 6px; font-size: 14px; }}
        textarea {{ resize: vertical; height: 100px; }}
        .checkbox-group, .radio-group {{ display: flex; flex-wrap: wrap; gap: 15px; }}
        .checkbox-item, .radio-item {{ display: flex; align-items: center; }}
        .checkbox-item input, .radio-item input {{ width: auto; margin-right: 8px; }}
        button {{ background: #007bff; color: white; padding: 15px 30px; border: none; border-radius: 6px; cursor: pointer; font-size: 16px; font-weight: 600; }}
        button:hover {{ background: #0056b3; }}
        .test-indicator {{ position: fixed; top: 10px; right: 10px; background: #dc3545; color: white; padding: 10px 20px; border-radius: 6px; font-weight: bold; }}
    </style>
</head>
<body>
    <div class="test-indicator">COMPLEX TEST FIXTURE</div>
    <div class="form-container">
        <h1>User Registration Form</h1>
        <form id="test-form">
            <div id="personal-info" class="section">
                <h3>Personal Information</h3>
                <div class="form-row">
                    <div class="form-group">
                        <label for="first-name">First Name:</label>
                        <input type="text" id="first-name" name="firstName" required>
                    </div>
                    <div class="form-group">
                        <label for="last-name">Last Name:</label>
                        <input type="text" id="last-name" name="lastName" required>
                    </div>
                </div>
                <div class="form-row">
                    <div class="form-group">
                        <label for="birthdate">Date of Birth:</label>
                        <input type="date" id="birthdate" name="birthdate">
                    </div>
                    <div class="form-group">
                        <label for="gender">Gender:</label>
                        <div class="radio-group">
                            <div class="radio-item">
                                <input type="radio" id="male" name="gender" value="male">
                                <label for="male">Male</label>
                            </div>
                            <div class="radio-item">
                                <input type="radio" id="female" name="gender" value="female">
                                <label for="female">Female</label>
                            </div>
                            <div class="radio-item">
                                <input type="radio" id="other" name="gender" value="other">
                                <label for="other">Other</label>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div id="contact-info" class="section">
                <h3>Contact Information</h3>
                <div class="form-row">
                    <div class="form-group">
                        <label for="email">Email Address:</label>
                        <input type="email" id="email" name="email" required>
                    </div>
                    <div class="form-group">
                        <label for="phone">Phone Number:</label>
                        <input type="tel" id="phone" name="phone">
                    </div>
                </div>
                <div class="form-row">
                    <div class="form-group">
                        <label for="country">Country:</label>
                        <select id="country" name="country">
                            <option value="">Select Country</option>
                            <option value="us">United States</option>
                            <option value="ca">Canada</option>
                            <option value="uk">United Kingdom</option>
                            <option value="au">Australia</option>
                            <option value="de">Germany</option>
                            <option value="fr">France</option>
                            <option value="jp">Japan</option>
                        </select>
                    </div>
                    <div class="form-group">
                        <label for="city">City:</label>
                        <input type="text" id="city" name="city">
                    </div>
                </div>
            </div>

            <div id="preferences" class="section">
                <h3>Preferences</h3>
                <div class="form-group">
                    <label>Interests:</label>
                    <div class="checkbox-group">
                        <div class="checkbox-item">
                            <input type="checkbox" id="tech" name="interests" value="technology">
                            <label for="tech">Technology</label>
                        </div>
                        <div class="checkbox-item">
                            <input type="checkbox" id="sports" name="interests" value="sports">
                            <label for="sports">Sports</label>
                        </div>
                        <div class="checkbox-item">
                            <input type="checkbox" id="music" name="interests" value="music">
                            <label for="music">Music</label>
                        </div>
                        <div class="checkbox-item">
                            <input type="checkbox" id="travel" name="interests" value="travel">
                            <label for="travel">Travel</label>
                        </div>
                    </div>
                </div>
                <div class="form-group">
                    <label for="comments">Additional Comments:</label>
                    <textarea id="comments" name="comments" placeholder="Tell us more about yourself..."></textarea>
                </div>
            </div>

            <button type="submit">Register Account</button>
        </form>
    </div>
</body>
</html>
        "#)
    }

    fn generate_data_table_html(&self, options: &TestFixtureOptions) -> String {
        let mut rows = String::new();
        for i in 1..=options.element_count {
            rows.push_str(&format!(r#"
                <tr>
                    <td>{}</td>
                    <td>User {}</td>
                    <td>user{}@example.com</td>
                    <td>{}</td>
                    <td><span class="status {}">{}</span></td>
                    <td>
                        <button class="btn-edit" onclick="editRow({})">Edit</button>
                        <button class="btn-delete" onclick="deleteRow({})">Delete</button>
                    </td>
                </tr>
            "#, i, i, i, 
                if i % 3 == 0 { "Admin" } else if i % 2 == 0 { "User" } else { "Guest" },
                if i % 4 == 0 { "inactive" } else { "active" },
                if i % 4 == 0 { "Inactive" } else { "Active" },
                i, i
            ));
        }

        format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Data Table Test Fixture</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; background: #f8f9fa; }}
        .table-container {{ max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 12px; box-shadow: 0 4px 20px rgba(0,0,0,0.1); }}
        table {{ width: 100%; border-collapse: collapse; margin-top: 20px; }}
        th {{ background: #343a40; color: white; padding: 15px; text-align: left; font-weight: 600; cursor: pointer; }}
        th:hover {{ background: #495057; }}
        td {{ padding: 12px 15px; border-bottom: 1px solid #dee2e6; }}
        tr:nth-child(even) {{ background: #f8f9fa; }}
        tr:hover {{ background: #e9ecef; }}
        .status.active {{ color: #28a745; font-weight: 600; }}
        .status.inactive {{ color: #dc3545; font-weight: 600; }}
        .btn-edit {{ background: #007bff; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; margin-right: 5px; }}
        .btn-delete {{ background: #dc3545; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; }}
        .btn-edit:hover {{ background: #0056b3; }}
        .btn-delete:hover {{ background: #c82333; }}
        .test-indicator {{ position: fixed; top: 10px; right: 10px; background: #17a2b8; color: white; padding: 10px 20px; border-radius: 6px; font-weight: bold; }}
        .table-controls {{ margin-bottom: 20px; }}
        .search-box {{ padding: 10px; border: 1px solid #ced4da; border-radius: 4px; width: 300px; }}
        .add-button {{ background: #28a745; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer; margin-left: 10px; }}
    </style>
</head>
<body>
    <div class="test-indicator">TABLE TEST FIXTURE</div>
    <div class="table-container">
        <h1>User Management Table</h1>
        <div class="table-controls">
            <input type="text" class="search-box" placeholder="Search users..." id="search-input">
            <button class="add-button" onclick="addUser()">Add User</button>
        </div>
        <table id="test-table">
            <thead>
                <tr>
                    <th class="sortable" onclick="sortTable(0)">ID ↕</th>
                    <th class="sortable" onclick="sortTable(1)">Name ↕</th>
                    <th class="sortable" onclick="sortTable(2)">Email ↕</th>
                    <th class="sortable" onclick="sortTable(3)">Role ↕</th>
                    <th class="sortable" onclick="sortTable(4)">Status ↕</th>
                    <th>Actions</th>
                </tr>
            </thead>
            <tbody>
                {rows}
            </tbody>
        </table>
    </div>
    <script>
        function editRow(id) {{ alert('Edit user ' + id); }}
        function deleteRow(id) {{ 
            if(confirm('Delete user ' + id + '?')) {{
                document.querySelector('tr:nth-child(' + (id + 1) + ')').remove();
            }}
        }}
        function addUser() {{ alert('Add new user functionality'); }}
        function sortTable(column) {{ alert('Sort by column ' + column); }}
        
        document.getElementById('search-input').addEventListener('input', function() {{
            const searchTerm = this.value.toLowerCase();
            const rows = document.querySelectorAll('tbody tr');
            rows.forEach(row => {{
                const text = row.textContent.toLowerCase();
                row.style.display = text.includes(searchTerm) ? '' : 'none';
            }});
        }});
    </script>
</body>
</html>
        "#, rows=rows)
    }

    fn generate_navigation_html(&self, options: &TestFixtureOptions) -> String {
        let mut nav_items = String::new();
        for i in 1..=options.element_count {
            if i <= 3 {
                // Create dropdown menus for first 3 items
                let menu_html = format!("
                    <li class=\"nav-item dropdown\">
                        <a href=\"#\" class=\"nav-link dropdown-toggle\">Menu {}</a>
                        <ul class=\"dropdown-menu\">
                            <li><a href=\"/menu{}/item1\">Submenu 1</a></li>
                            <li><a href=\"/menu{}/item2\">Submenu 2</a></li>
                            <li><a href=\"/menu{}/item3\">Submenu 3</a></li>
                        </ul>
                    </li>
                ", i, i, i, i);
                nav_items.push_str(&menu_html);
            } else {
                let page_html = format!("
                    <li class=\"nav-item\">
                        <a href=\"/page{}\" class=\"nav-link\">Page {}</a>
                    </li>
                ", i, i);
                nav_items.push_str(&page_html);
            }
        }

        format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Navigation Test Fixture</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 0; background: #f8f9fa; }}
        .navbar {{ background: #343a40; padding: 1rem 2rem; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
        .nav-container {{ max-width: 1200px; margin: 0 auto; display: flex; justify-content: space-between; align-items: center; }}
        .nav-brand {{ color: #fff; font-size: 1.5rem; font-weight: bold; text-decoration: none; }}
        .nav-menu {{ display: flex; list-style: none; margin: 0; padding: 0; }}
        .nav-item {{ position: relative; }}
        .nav-link {{ color: #fff; text-decoration: none; padding: 0.5rem 1rem; display: block; transition: background 0.3s; }}
        .nav-link:hover {{ background: #495057; border-radius: 4px; }}
        .nav-link.active {{ background: #007bff; border-radius: 4px; }}
        .dropdown-menu {{ position: absolute; top: 100%; left: 0; background: #fff; border: 1px solid #dee2e6; border-radius: 4px; min-width: 200px; box-shadow: 0 4px 20px rgba(0,0,0,0.1); display: none; z-index: 1000; }}
        .dropdown:hover .dropdown-menu {{ display: block; }}
        .dropdown-menu li {{ list-style: none; }}
        .dropdown-menu a {{ color: #333; padding: 0.7rem 1rem; display: block; text-decoration: none; transition: background 0.3s; }}
        .dropdown-menu a:hover {{ background: #f8f9fa; }}
        .content {{ max-width: 1200px; margin: 2rem auto; padding: 0 2rem; }}
        .test-indicator {{ position: fixed; top: 10px; right: 10px; background: #fd7e14; color: white; padding: 10px 20px; border-radius: 6px; font-weight: bold; z-index: 1001; }}
        .mobile-menu-toggle {{ display: none; background: none; border: none; color: #fff; font-size: 1.5rem; cursor: pointer; }}
        
        @media (max-width: 768px) {{
            .mobile-menu-toggle {{ display: block; }}
            .nav-menu {{ display: none; flex-direction: column; position: absolute; top: 100%; left: 0; right: 0; background: #343a40; border-top: 1px solid #495057; }}
            .nav-menu.active {{ display: flex; }}
            .nav-item {{ width: 100%; }}
            .dropdown-menu {{ position: static; display: block; box-shadow: none; border: none; background: #495057; }}
        }}
    </style>
</head>
<body>
    <div class="test-indicator">NAV TEST FIXTURE</div>
    <nav class="navbar">
        <div class="nav-container">
            <a href="/" class="nav-brand">Test Site</a>
            <button class="mobile-menu-toggle" onclick="toggleMobileMenu()">☰</button>
            <ul id="main-nav" class="nav-menu">
                <li class="nav-item">
                    <a href="/" class="nav-link active">Home</a>
                </li>
                {nav_items}
                <li class="nav-item">
                    <a href="/contact" class="nav-link">Contact</a>
                </li>
            </ul>
        </div>
    </nav>
    
    <div class="content">
        <h1>Navigation Test Page</h1>
        <p>This is a test fixture for navigation components. Use this page to test navigation interactions, dropdown menus, and responsive behavior.</p>
        
        <h2>Available Test Selectors:</h2>
        <ul>
            <li><code>#main-nav</code> - Main navigation container</li>
            <li><code>.nav-item</code> - Individual navigation items</li>
            <li><code>.nav-link</code> - Navigation links</li>
            <li><code>.dropdown</code> - Dropdown containers</li>
            <li><code>.dropdown-menu</code> - Dropdown menus</li>
            <li><code>.active</code> - Active navigation item</li>
            <li><code>.nav-brand</code> - Site brand/logo</li>
        </ul>
    </div>
    
    <script>
        function toggleMobileMenu() {{
            const menu = document.getElementById('main-nav');
            menu.classList.toggle('active');
        }}
        
        // Add click handlers for navigation
        document.querySelectorAll('.nav-link').forEach(link => {{
            link.addEventListener('click', function(e) {{
                if (this.getAttribute('href').startsWith('#') || this.classList.contains('dropdown-toggle')) {{
                    e.preventDefault();
                }}
                
                // Update active state
                document.querySelectorAll('.nav-link').forEach(l => l.classList.remove('active'));
                this.classList.add('active');
                
                console.log('Navigated to:', this.textContent);
            }});
        }});
    </script>
</body>
</html>
        "#, nav_items=nav_items)
    }
}

#[async_trait]
impl Tool for CreateTestFixtureTool {
    type Input = CreateTestFixtureInput;
    type Output = CreateTestFixtureOutput;
    
    fn name(&self) -> &str {
        "create_test_fixture"
    }
    
    fn description(&self) -> &str {
        "Create synthetic HTML test fixtures for testing browser automation tools"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::AdvancedAutomation
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        info!("Creating test fixture: {:?}", input.fixture_type);
        
        match input.fixture_type {
            TestFixtureType::SimpleForm => self.create_simple_form(&input.options).await,
            TestFixtureType::ComplexForm => self.create_complex_form(&input.options).await,
            TestFixtureType::DataTable => self.create_data_table(&input.options).await,
            TestFixtureType::NavigationMenu => self.create_navigation_menu(&input.options).await,
            TestFixtureType::ContentPage => {
                // TODO: Implement content page fixture
                Err(anyhow!("Content page fixture not yet implemented"))
            }
            TestFixtureType::LoadingStates => {
                // TODO: Implement loading states fixture
                Err(anyhow!("Loading states fixture not yet implemented"))
            }
            TestFixtureType::ErrorPage => {
                // TODO: Implement error page fixture
                Err(anyhow!("Error page fixture not yet implemented"))
            }
            TestFixtureType::Interactive => {
                // TODO: Implement interactive elements fixture
                Err(anyhow!("Interactive fixture not yet implemented"))
            }
        }
    }
    
    async fn validate_input(&self, input: &Self::Input) -> Result<()> {
        if input.options.element_count == 0 {
            return Err(anyhow!("Element count must be greater than 0"));
        }
        if input.options.element_count > 1000 {
            return Err(anyhow!("Element count cannot exceed 1000"));
        }
        Ok(())
    }
}