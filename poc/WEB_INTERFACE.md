# RainbowBrowserAI Web Dashboard

## Overview

RainbowBrowserAI includes a beautiful web dashboard interface for controlling the browser automation system. The dashboard provides an intuitive UI for executing commands, managing browser sessions, and monitoring system status.

## Starting the Service

### Windows
```bash
start.bat
```
The dashboard will automatically open at http://localhost:3000

### Linux/Mac
```bash
./start.sh
```
The dashboard will automatically open at http://localhost:3000

### Manual Start
```bash
# Start ChromeDriver first
chromedriver --port=9515

# In another terminal, start the server
cargo run --bin rainbow-poc -- serve --port 3000
```

## Dashboard Features

### Main Dashboard (`/` or `/index.html`)
The main dashboard provides:

- **Command Interface**: Natural language command input for browser automation
- **Browse Tab**: Direct URL navigation with screenshot capabilities
- **Workflow Tab**: Visual workflow builder for complex automation sequences
- **Sessions Tab**: Manage multiple browser sessions
- **Plugins Tab**: Manage and configure plugins
- **Metrics Tab**: View performance metrics and costs
- **Logs Tab**: Real-time log monitoring
- **Settings Tab**: Configure API keys and preferences

### Production Dashboard (`/production.html`)
A specialized dashboard for production monitoring:

- **System Health Monitoring**: Real-time health status
- **Performance Metrics**: CPU, memory, and response time graphs
- **Active Sessions**: Monitor all active browser sessions
- **Cost Tracking**: Track API usage and costs
- **Security Monitoring**: Track authentication and security events
- **Alert Management**: Configure and view system alerts

## Dashboard Components

### Status Indicators
- 🟢 Green: System healthy and connected
- 🟡 Yellow: Warning or degraded performance
- 🔴 Red: Error or disconnected

### Real-time Updates
The dashboard uses Server-Sent Events (SSE) for real-time updates:
- Command execution progress
- Browser navigation events
- Screenshot captures
- Error notifications
- Cost updates

## API Endpoints

The dashboard communicates with these API endpoints:

- `GET /health` - System health check
- `POST /api/navigate` - Navigate to URL
- `POST /api/ask` - Execute natural language command
- `GET /metrics` - Get system metrics
- `GET /cost` - Get cost information
- `GET /events` - SSE endpoint for real-time updates

## Customization

### Themes
The dashboard supports both light and dark themes, controlled via the settings panel.

### Layout
The dashboard is fully responsive and works on:
- Desktop (1920x1080 and higher)
- Tablet (768px - 1024px)
- Mobile (320px - 768px)

## Files Structure

```
static/
├── index.html              # Main dashboard
├── styles.css              # Main styles
├── app.js                  # Dashboard application logic
├── production.html         # Production monitoring dashboard
├── production-styles.css   # Production dashboard styles
└── production-dashboard.js # Production dashboard logic
```

## Browser Compatibility

The dashboard is tested and works on:
- Chrome/Chromium 90+
- Firefox 88+
- Safari 14+
- Edge 90+

## Security

The dashboard includes:
- CORS protection
- XSS prevention
- Input validation
- Rate limiting on API endpoints

## Troubleshooting

### Dashboard not loading
1. Check if the server is running: `curl http://localhost:3000/health`
2. Check browser console for errors (F12)
3. Ensure ChromeDriver is running on port 9515

### Real-time updates not working
1. Check if SSE endpoint is accessible: `curl http://localhost:3000/events`
2. Check browser network tab for SSE connection
3. Ensure no proxy/firewall is blocking SSE

### Commands not executing
1. Verify LLM API key is configured
2. Check server logs for errors
3. Ensure ChromeDriver is properly connected

## Development

To modify the dashboard:

1. Edit files in `static/` directory
2. No build process required - changes are live
3. Refresh browser to see changes
4. Use browser dev tools for debugging

## Screenshots

The dashboard automatically displays screenshots when:
- Navigation commands include screenshot flag
- Screenshot button is clicked
- Workflow steps include screenshot action

Screenshots are displayed inline and can be downloaded.

## Future Enhancements

Planned features:
- [ ] Multi-language support
- [ ] Advanced workflow templates
- [ ] Visual regression testing
- [ ] Performance profiling graphs
- [ ] Export/import workflow configurations
- [ ] Collaborative session sharing
- [ ] Mobile app companion

## Support

For issues or questions about the web dashboard:
1. Check the browser console for errors
2. Review server logs
3. Open an issue on GitHub with details