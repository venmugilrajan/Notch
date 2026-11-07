#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConverterCategory {
    Length,
    Weight,
    Temperature,
    Currency,
    Data,
}

#[derive(Clone, Debug)]
pub struct ConverterState {
    pub category: ConverterCategory,
    pub input_str: String,
    pub from_unit: usize,
    pub to_unit: usize,
}

impl ConverterState {
    pub fn new() -> Self {
        Self {
            category: ConverterCategory::Length,
            input_str: "10".to_string(),
            from_unit: 0,
            to_unit: 1,
        }
    }

    pub fn categories() -> [&'static str; 5] {
        ["Length", "Weight", "Temperature", "Currency", "Data"]
    }

    pub fn units(&self) -> Vec<&'static str> {
        match self.category {
            ConverterCategory::Length => vec!["Meters (m)", "Kilometers (km)", "Centimeters (cm)", "Millimeters (mm)", "Feet (ft)", "Inches (in)", "Yards (yd)", "Miles (mi)"],
            ConverterCategory::Weight => vec!["Kilograms (kg)", "Grams (g)", "Pounds (lbs)", "Ounces (oz)", "Milligrams (mg)", "Metric Tons (t)"],
            ConverterCategory::Temperature => vec!["Celsius (°C)", "Fahrenheit (°F)", "Kelvin (K)"],
            ConverterCategory::Currency => vec!["USD ($)", "EUR (€)", "INR (₹)", "GBP (£)", "JPY (¥)", "CAD ($)", "AUD ($)", "CNY (¥)"],
            ConverterCategory::Data => vec!["Megabytes (MB)", "Gigabytes (GB)", "Kilobytes (KB)", "Terabytes (TB)", "Bytes (B)", "Petabytes (PB)"],
        }
    }

    pub fn push_digit(&mut self, ch: char) {
        if ch == '.' {
            if !self.input_str.contains('.') {
                if self.input_str.is_empty() {
                    self.input_str.push('0');
                }
                self.input_str.push('.');
            }
        } else if ch == 'C' {
            self.input_str = "0".to_string();
        } else if ch == 'B' { // backspace
            if !self.input_str.is_empty() {
                self.input_str.pop();
            }
            if self.input_str.is_empty() {
                self.input_str = "0".to_string();
            }
        } else if ch == 'S' { // sign +/-
            if self.input_str.starts_with('-') {
                self.input_str.remove(0);
            } else if self.input_str != "0" {
                self.input_str.insert(0, '-');
            }
        } else if ch.is_ascii_digit() {
            if self.input_str == "0" {
                self.input_str = ch.to_string();
            } else if self.input_str.len() < 12 {
                self.input_str.push(ch);
            }
        }
    }

    pub fn parsed_value(&self) -> f64 {
        self.input_str.parse::<f64>().unwrap_or(0.0)
    }

    pub fn convert(&self) -> (f64, &'static str) {
        let val = self.parsed_value();
        match self.category {
            ConverterCategory::Length => {
                let meters = match self.from_unit {
                    0 => val,                 // Meters
                    1 => val * 1000.0,        // Kilometers
                    2 => val * 0.01,          // Centimeters
                    3 => val * 0.001,         // Millimeters
                    4 => val * 0.3048,        // Feet
                    5 => val * 0.0254,        // Inches
                    6 => val * 0.9144,        // Yards
                    7 => val * 1609.344,      // Miles
                    _ => val,
                };
                let result = match self.to_unit {
                    0 => meters,
                    1 => meters / 1000.0,
                    2 => meters / 0.01,
                    3 => meters / 0.001,
                    4 => meters / 0.3048,
                    5 => meters / 0.0254,
                    6 => meters / 0.9144,
                    7 => meters / 1609.344,
                    _ => meters,
                };
                let unit_label = match self.to_unit {
                    0 => "m", 1 => "km", 2 => "cm", 3 => "mm", 4 => "ft", 5 => "in", 6 => "yd", 7 => "mi", _ => "",
                };
                (result, unit_label)
            }
            ConverterCategory::Weight => {
                let kg = match self.from_unit {
                    0 => val,                 // kg
                    1 => val / 1000.0,        // g
                    2 => val * 0.45359237,    // lbs
                    3 => val * 0.0283495,     // oz
                    4 => val / 1_000_000.0,   // mg
                    5 => val * 1000.0,        // metric tons
                    _ => val,
                };
                let result = match self.to_unit {
                    0 => kg,
                    1 => kg * 1000.0,
                    2 => kg / 0.45359237,
                    3 => kg / 0.0283495,
                    4 => kg * 1_000_000.0,
                    5 => kg / 1000.0,
                    _ => kg,
                };
                let unit_label = match self.to_unit {
                    0 => "kg", 1 => "g", 2 => "lbs", 3 => "oz", 4 => "mg", 5 => "t", _ => "",
                };
                (result, unit_label)
            }
            ConverterCategory::Temperature => {
                let celsius = match self.from_unit {
                    0 => val,
                    1 => (val - 32.0) * 5.0 / 9.0,
                    2 => val - 273.15,
                    _ => val,
                };
                let result = match self.to_unit {
                    0 => celsius,
                    1 => (celsius * 9.0 / 5.0) + 32.0,
                    2 => celsius + 273.15,
                    _ => celsius,
                };
                let unit_label = match self.to_unit {
                    0 => "°C", 1 => "°F", 2 => "K", _ => "",
                };
                (result, unit_label)
            }
            ConverterCategory::Currency => {
                let usd = match self.from_unit {
                    0 => val,
                    1 => val / 0.92,
                    2 => val / 84.5,
                    3 => val / 0.78,
                    4 => val / 152.0,
                    5 => val / 1.35,
                    6 => val / 1.50,
                    7 => val / 7.20,
                    _ => val,
                };
                let result = match self.to_unit {
                    0 => usd,
                    1 => usd * 0.92,
                    2 => usd * 84.5,
                    3 => usd * 0.78,
                    4 => usd * 152.0,
                    5 => usd * 1.35,
                    6 => usd * 1.50,
                    7 => usd * 7.20,
                    _ => usd,
                };
                let unit_label = match self.to_unit {
                    0 => "USD", 1 => "EUR", 2 => "INR", 3 => "GBP", 4 => "JPY", 5 => "CAD", 6 => "AUD", 7 => "CNY", _ => "",
                };
                (result, unit_label)
            }
            ConverterCategory::Data => {
                let mb = match self.from_unit {
                    0 => val,                             // MB
                    1 => val * 1024.0,                    // GB
                    2 => val / 1024.0,                    // KB
                    3 => val * 1024.0 * 1024.0,           // TB
                    4 => val / (1024.0 * 1024.0),         // B
                    5 => val * 1024.0 * 1024.0 * 1024.0,  // PB
                    _ => val,
                };
                let result = match self.to_unit {
                    0 => mb,
                    1 => mb / 1024.0,
                    2 => mb * 1024.0,
                    3 => mb / (1024.0 * 1024.0),
                    4 => mb * 1024.0 * 1024.0,
                    5 => mb / (1024.0 * 1024.0 * 1024.0),
                    _ => mb,
                };
                let unit_label = match self.to_unit {
                    0 => "MB", 1 => "GB", 2 => "KB", 3 => "TB", 4 => "B", 5 => "PB", _ => "",
                };
                (result, unit_label)
            }
        }
    }
}
