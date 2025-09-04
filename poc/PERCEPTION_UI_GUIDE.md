# Perception UI Guide

## Overview
The perception module UI has been successfully integrated into the RainbowBrowserAI dashboard at http://localhost:3001

## How to Access

1. **Open the Dashboard**: Navigate to http://localhost:3001 in your web browser
2. **Go to Command Tab**: The perception section is located in the "Command" tab (the first tab)
3. **Look for Perception Analysis Section**: You'll see it below the command output area

## Perception Features

The perception UI provides three main analysis capabilities:

### 1. Analyze Current Page
- **Button**: 🔍 Analyze Current Page
- **Function**: Classifies the current webpage type (e.g., Login, E-commerce, Blog, etc.)
- **API Endpoint**: `/api/perception` with `action: "classify"`

### 2. Extract Page Data  
- **Button**: 📊 Extract Page Data
- **Function**: Extracts structured data from the page (headings, links, forms, etc.)
- **API Endpoint**: `/api/perception` with `action: "extract_data"`

### 3. Find Elements
- **Button**: 🖱️ Find Elements
- **Function**: Finds specific elements on the page (prompts for element type)
- **API Endpoint**: `/api/perception` with `action: "find_element"`

## How to Use

1. **Navigate to a webpage first**:
   - Use the Browse tab to navigate to any URL
   - Or use the Command tab with a navigate command

2. **Click any perception button**:
   - The system will analyze the current page
   - Results will display below the buttons

3. **View the results**:
   - **Page Type**: Shows the classification of the page
   - **Elements Found**: Lists discovered elements with details
   - **Extracted Data**: Shows structured data from the page

## Visual Indicators

- **Loading State**: Spinner animation while analyzing
- **Success State**: Green notification and results display
- **Error State**: Red error message if analysis fails
- **Empty State**: Info message when no page is loaded

## Technical Details

- The perception module uses ChromeDriver to analyze page DOM
- Works in both mock mode and real browser mode
- Provides confidence scores for classifications
- Extracts semantic information from page structure

## Testing the Feature

1. Navigate to http://localhost:3001
2. Go to the Browse tab and navigate to any website (e.g., https://github.com)
3. Return to the Command tab
4. Click any of the three perception buttons
5. View the analysis results

## API Integration

The perception UI communicates with these endpoints:
- `POST /api/perception` - Main perception endpoint
- `POST /api/command` - For navigation commands
- `GET /api/health` - To check service status

## Current Status

✅ Perception UI is now visible by default on the dashboard
✅ All three analysis functions are working
✅ Results display properly with formatted output
✅ Error handling and loading states implemented