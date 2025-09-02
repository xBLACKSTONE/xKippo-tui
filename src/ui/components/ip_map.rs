use std::collections::{HashMap, HashSet};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};

/// IP map model for mapping IPs to geographical locations
pub struct IpMapModel {
    /// Mapping of IPs to coordinates (latitude, longitude)
    pub ip_coordinates: HashMap<String, (f64, f64)>,
    /// IPs to highlight
    pub highlighted_ips: HashSet<String>,
    /// Map dimensions
    pub width: usize,
    pub height: usize,
    /// Map boundaries
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

impl Default for IpMapModel {
    fn default() -> Self {
        Self {
            ip_coordinates: HashMap::new(),
            highlighted_ips: HashSet::new(),
            width: 80,
            height: 24,
            min_lat: -90.0,
            max_lat: 90.0,
            min_lon: -180.0,
            max_lon: 180.0,
        }
    }
}

impl IpMapModel {
    /// Add an IP with coordinates
    pub fn add_ip(&mut self, ip: String, lat: f64, lon: f64) {
        self.ip_coordinates.insert(ip, (lat, lon));
    }
    
    /// Highlight an IP
    pub fn highlight_ip(&mut self, ip: String) {
        self.highlighted_ips.insert(ip);
    }
    
    /// Clear highlighted IPs
    pub fn clear_highlights(&mut self) {
        self.highlighted_ips.clear();
    }
    
    /// Get coordinate for the world map
    fn get_map_coordinate(&self, lat: f64, lon: f64) -> (usize, usize) {
        let x = ((lon - self.min_lon) / (self.max_lon - self.min_lon) * (self.width as f64 - 1.0)) as usize;
        let y = ((self.max_lat - lat) / (self.max_lat - self.min_lat) * (self.height as f64 - 1.0)) as usize;
        (x, y)
    }
}

/// World map widget state
pub struct IpMapState {
    /// Selected IP
    pub selected_ip: Option<String>,
    /// Map scroll position
    pub scroll: (u16, u16),
}

impl Default for IpMapState {
    fn default() -> Self {
        Self {
            selected_ip: None,
            scroll: (0, 0),
        }
    }
}

/// Widget for displaying IPs on a world map
pub struct IpMapWidget<'a> {
    /// Block to wrap the widget in
    pub block: Option<Block<'a>>,
    /// Map model
    pub model: &'a IpMapModel,
    /// Normal style
    pub style: Style,
    /// Style for highlighted IPs
    pub highlight_style: Style,
}

impl<'a> IpMapWidget<'a> {
    /// Create a new IP map widget
    pub fn new(model: &'a IpMapModel) -> Self {
        Self {
            block: None,
            model,
            style: Style::default().fg(Color::Yellow),
            highlight_style: Style::default().fg(Color::Red),
        }
    }
    
    /// Set the block for the widget
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
    
    /// Set the normal style
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
    
    /// Set the highlight style
    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }
}

impl<'a> StatefulWidget for IpMapWidget<'a> {
    type State = IpMapState;
    
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Render block if present
        let area = match self.block {
            Some(b) => {
                let inner = b.inner(area);
                b.render(area, buf);
                inner
            }
            None => area,
        };
        
        // Skip if area is too small
        if area.width < 3 || area.height < 3 {
            return;
        }
        
        // Create a simple ASCII world map
        let mut map = vec![vec![' '; area.width as usize]; area.height as usize];
        
        // Draw a simple world outline
        for y in 0..area.height as usize {
            for x in 0..area.width as usize {
                let lat = self.model.max_lat - (y as f64 / area.height as f64) * (self.model.max_lat - self.model.min_lat);
                let lon = self.model.min_lon + (x as f64 / area.width as f64) * (self.model.max_lon - self.model.min_lon);
                
                // Very simple map outline (this would be improved in a real implementation)
                if lat > 60.0 || lat < -60.0 {
                    map[y][x] = '.';
                }
                
                // Continents (extremely simplified)
                if (lat > 20.0 && lat < 70.0 && lon > -10.0 && lon < 40.0) || // Europe
                   (lat > 0.0 && lat < 70.0 && lon > 60.0 && lon < 140.0) || // Asia
                   (lat > -40.0 && lat < 40.0 && lon > -20.0 && lon < 50.0) || // Africa
                   (lat > -10.0 && lat < 15.0 && lon > 100.0 && lon < 140.0) || // Southeast Asia
                   (lat > -40.0 && lat < 10.0 && lon > 110.0 && lon < 180.0) || // Australia
                   (lat > 15.0 && lat < 70.0 && lon > -170.0 && lon < -50.0) || // North America
                   (lat > -60.0 && lat < 15.0 && lon > -80.0 && lon < -35.0) // South America
                {
                    map[y][x] = '█';
                }
            }
        }
        
        // Plot IPs on the map
        for (ip, (lat, lon)) in &self.model.ip_coordinates {
            let (x, y) = self.model.get_map_coordinate(*lat, *lon);
            
            // Skip if out of bounds
            if x >= area.width as usize || y >= area.height as usize {
                continue;
            }
            
            // Different character for highlighted IPs
            if self.model.highlighted_ips.contains(ip) || state.selected_ip.as_ref() == Some(ip) {
                map[y][x] = '★';
            } else {
                map[y][x] = '●';
            }
        }
        
        // Render the map
        for (y, row) in map.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                let style = if cell == '★' {
                    self.highlight_style
                } else if cell == '●' {
                    self.style
                } else if cell == '█' {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                
                buf.get_mut(area.x + x as u16, area.y + y as u16)
                    .set_char(cell)
                    .set_style(style);
            }
        }
    }
}

impl<'a> Widget for IpMapWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = IpMapState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}