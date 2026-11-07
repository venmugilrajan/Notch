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
            ConverterCategory::Length => vec!["Meters (m)", "Feet (ft)", "Inches (in)", "Kilometers (km)", "Miles (mi)"],
            ConverterCategory::Weight => vec!["Kilograms (kg)", "Pounds (lbs)", "Grams (g)", "Ounces (oz)"],
            ConverterCategory::Temperature => vec!["Celsius (°C)", "Fahrenheit (°F)", "Kelvin (K)"],
            ConverterCategory::Currency => vec!["USD ($)", "EUR (€)", "INR (₹)", "GBP (£)", "JPY (¥)"],
            ConverterCategory::Data => vec!["Megabytes (MB)", "Gigabytes (GB)", "Kilobytes (KB)", "Terabytes (TB)"],
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
                    0 => val,
                    1 => val * 0.3048,
                    2 => val * 0.0254,
                    3 => val * 1000.0,
                    4 => val * 1609.34,
                    _ => val,
                };
                let result = match self.to_unit {
                    0 => meters,
                    1 => meters / 0.3048,
                    2 => meters / 0.0254,
                    3 => meters / 1000.0,
                    4 => meters / 1609.34,
                    _ => meters,
                };
                let unit_label = match self.to_unit {
                    0 => "m", 1 => "ft", 2 => "in", 3 => "km", 4 => "mi", _ => "",
                };
                (result, unit_label)
            }
            ConverterCategory::Weight => {
                let kg = match self.from_unit {
                    0 => val,
                    1 => val * 0.453592,
                    2 => val / 1000.0,
                    3 => val * 0.0283495,
                    _ => val,
                };
                let result = match self.to_unit {
                    0 => kg,
                    1 => kg / 0.453592,
                    2 => kg * 1000.0,
                    3 => kg / 0.0283495,
                    _ => kg,
                };
                let unit_label = match self.to_unit {
                    0 => "kg", 1 => "lbs", 2 => "g", 3 => "oz", _ => "",
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
                    _ => val,
                };
                let result = match self.to_unit {
                    0 => usd,
                    1 => usd * 0.92,
                    2 => usd * 84.5,
                    3 => usd * 0.78,
                    4 => usd * 152.0,
                    _ => usd,
                };
                let unit_label = match self.to_unit {
                    0 => "USD", 1 => "EUR", 2 => "INR", 3 => "GBP", 4 => "JPY", _ => "",
                };
                (result, unit_label)
            }
            ConverterCategory::Data => {
                let mb = match self.from_unit {
                    0 => val,
                    1 => val * 1024.0,
                    2 => val / 1024.0,
                    3 => val * 1024.0 * 1024.0,
                    _ => val,
                };
                let result = match self.to_unit {
                    0 => mb,
                    1 => mb / 1024.0,
                    2 => mb * 1024.0,
                    3 => mb / (1024.0 * 1024.0),
                    _ => mb,
                };
                let unit_label = match self.to_unit {
                    0 => "MB", 1 => "GB", 2 => "KB", 3 => "TB", _ => "",
                };
                (result, unit_label)
            }
        }
    }
}
