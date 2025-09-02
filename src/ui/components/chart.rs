use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{
        Axis, Block, Chart as RatatuiChart, Dataset, GraphType, Widget,
    },
};

/// Data point with time and value
#[derive(Debug, Clone, Copy)]
pub struct DataPoint {
    /// X value (typically time)
    pub x: f64,
    /// Y value
    pub y: f64,
}

/// Chart widget wrapper for easier use
pub struct ChartWidget<'a> {
    /// Chart title
    pub title: &'a str,
    /// X-axis title
    pub x_title: &'a str,
    /// Y-axis title
    pub y_title: &'a str,
    /// Data series
    pub datasets: Vec<(String, Vec<DataPoint>, Color)>,
    /// X-axis bounds
    pub x_bounds: [f64; 2],
    /// Y-axis bounds
    pub y_bounds: [f64; 2],
    /// Block to wrap the chart in
    pub block: Option<Block<'a>>,
    /// Chart type
    pub chart_type: GraphType,
    /// Line style
    pub line_style: Style,
}

impl<'a> Default for ChartWidget<'a> {
    fn default() -> Self {
        Self {
            title: "",
            x_title: "",
            y_title: "",
            datasets: Vec::new(),
            x_bounds: [0.0, 100.0],
            y_bounds: [0.0, 100.0],
            block: None,
            chart_type: GraphType::Line,
            line_style: Style::default(),
        }
    }
}

impl<'a> ChartWidget<'a> {
    /// Create a new chart widget
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set chart title
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }
    
    /// Set X-axis title
    pub fn x_title(mut self, title: &'a str) -> Self {
        self.x_title = title;
        self
    }
    
    /// Set Y-axis title
    pub fn y_title(mut self, title: &'a str) -> Self {
        self.y_title = title;
        self
    }
    
    /// Add a data series
    pub fn add_dataset(mut self, name: String, data: Vec<DataPoint>, color: Color) -> Self {
        self.datasets.push((name, data, color));
        self
    }
    
    /// Set X-axis bounds
    pub fn x_bounds(mut self, bounds: [f64; 2]) -> Self {
        self.x_bounds = bounds;
        self
    }
    
    /// Set Y-axis bounds
    pub fn y_bounds(mut self, bounds: [f64; 2]) -> Self {
        self.y_bounds = bounds;
        self
    }
    
    /// Set block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
    
    /// Set chart type
    pub fn chart_type(mut self, chart_type: GraphType) -> Self {
        self.chart_type = chart_type;
        self
    }
    
    /// Set line style
    pub fn line_style(mut self, style: Style) -> Self {
        self.line_style = style;
        self
    }
    
    /// Auto-calculate bounds from data
    pub fn auto_bounds(mut self) -> Self {
        if self.datasets.is_empty() {
            return self;
        }
        
        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;
        
        for (_, data, _) in &self.datasets {
            for point in data {
                min_x = min_x.min(point.x);
                max_x = max_x.max(point.x);
                min_y = min_y.min(point.y);
                max_y = max_y.max(point.y);
            }
        }
        
        // Add some padding
        let x_padding = (max_x - min_x) * 0.05;
        let y_padding = (max_y - min_y) * 0.1;
        
        self.x_bounds = [min_x - x_padding, max_x + x_padding];
        self.y_bounds = [min_y - y_padding, max_y + y_padding];
        
        self
    }
}

impl<'a> Widget for ChartWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Convert data points to format expected by ratatui
        let datasets: Vec<Dataset> = self.datasets
            .iter()
            .map(|(name, data, color)| {
                Dataset::default()
                    .name(name)
                    .marker(symbols::Marker::Dot)
                    .style(Style::default().fg(*color))
                    .graph_type(self.chart_type)
                    .data(&data.iter()
                        .map(|point| (point.x, point.y))
                        .collect::<Vec<(f64, f64)>>())
            })
            .collect();
        
        // Create X-axis
        let x_labels = vec![
            Span::raw(format!("{:.1}", self.x_bounds[0])),
            Span::raw(format!("{:.1}", (self.x_bounds[0] + self.x_bounds[1]) / 2.0)),
            Span::raw(format!("{:.1}", self.x_bounds[1])),
        ];
        
        let y_labels = vec![
            Span::raw(format!("{:.1}", self.y_bounds[0])),
            Span::raw(format!("{:.1}", (self.y_bounds[0] + self.y_bounds[1]) / 2.0)),
            Span::raw(format!("{:.1}", self.y_bounds[1])),
        ];
        
        // Create the chart
        let chart = RatatuiChart::new(datasets)
            .block(self.block.unwrap_or_else(|| Block::default().title(self.title)))
            .x_axis(
                Axis::default()
                    .title(Span::styled(self.x_title, Style::default().fg(Color::Gray)))
                    .style(Style::default().fg(Color::Gray))
                    .bounds(self.x_bounds)
                    .labels(x_labels)
            )
            .y_axis(
                Axis::default()
                    .title(Span::styled(self.y_title, Style::default().fg(Color::Gray)))
                    .style(Style::default().fg(Color::Gray))
                    .bounds(self.y_bounds)
                    .labels(y_labels)
            );
        
        chart.render(area, buf);
    }
}