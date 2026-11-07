use windows::core::*;
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;

pub struct IconPainter {
    stroke_style: ID2D1StrokeStyle,
}

impl IconPainter {
    pub fn new(factory: &ID2D1Factory) -> Result<Self> {
        let props = D2D1_STROKE_STYLE_PROPERTIES {
            startCap: D2D1_CAP_STYLE_ROUND,
            endCap: D2D1_CAP_STYLE_ROUND,
            dashCap: D2D1_CAP_STYLE_ROUND,
            lineJoin: D2D1_LINE_JOIN_ROUND,
            miterLimit: 10.0,
            dashStyle: D2D1_DASH_STYLE_SOLID,
            dashOffset: 0.0,
        };
        let stroke_style = unsafe { factory.CreateStrokeStyle(&props, None)? };
        Ok(Self { stroke_style })
    }

    pub fn draw_icon(
        &self,
        target: &ID2D1DCRenderTarget,
        factory: &ID2D1Factory,
        index: usize,
        cx: f32,
        cy: f32,
        brush: &ID2D1SolidColorBrush,
    ) -> Result<()> {
        unsafe {
            let stroke = &self.stroke_style;
            let stroke_w = 1.5;

            match index {
                0 => {
                    // Home: House outline
                    let geom = factory.CreatePathGeometry()?;
                    let sink = geom.Open()?;
                    sink.BeginFigure(D2D_POINT_2F { x: cx - 7.0, y: cy + 7.0 }, D2D1_FIGURE_BEGIN_HOLLOW);
                    sink.AddLine(D2D_POINT_2F { x: cx - 7.0, y: cy - 1.0 });
                    sink.AddLine(D2D_POINT_2F { x: cx, y: cy - 7.5 });
                    sink.AddLine(D2D_POINT_2F { x: cx + 7.0, y: cy - 1.0 });
                    sink.AddLine(D2D_POINT_2F { x: cx + 7.0, y: cy + 7.0 });
                    sink.AddLine(D2D_POINT_2F { x: cx + 2.5, y: cy + 7.0 });
                    sink.AddLine(D2D_POINT_2F { x: cx + 2.5, y: cy + 2.0 });
                    sink.AddLine(D2D_POINT_2F { x: cx - 2.5, y: cy + 2.0 });
                    sink.AddLine(D2D_POINT_2F { x: cx - 2.5, y: cy + 7.0 });
                    sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                    sink.Close()?;
                    target.DrawGeometry(&geom, brush, stroke_w, Some(stroke));
                }
                1 => {
                    // System: Microchip
                    let rect = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F { left: cx - 5.0, top: cy - 5.0, right: cx + 5.0, bottom: cy + 5.0 },
                        radiusX: 2.0,
                        radiusY: 2.0,
                    };
                    target.DrawRoundedRectangle(&rect, brush, stroke_w, Some(stroke));
                    // Pins
                    target.DrawLine(D2D_POINT_2F { x: cx - 7.5, y: cy - 2.5 }, D2D_POINT_2F { x: cx - 5.0, y: cy - 2.5 }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx - 7.5, y: cy + 2.5 }, D2D_POINT_2F { x: cx - 5.0, y: cy + 2.5 }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx + 5.0, y: cy - 2.5 }, D2D_POINT_2F { x: cx + 7.5, y: cy - 2.5 }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx + 5.0, y: cy + 2.5 }, D2D_POINT_2F { x: cx + 7.5, y: cy + 2.5 }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx - 2.5, y: cy - 7.5 }, D2D_POINT_2F { x: cx - 2.5, y: cy - 5.0 }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx + 2.5, y: cy - 7.5 }, D2D_POINT_2F { x: cx + 2.5, y: cy - 5.0 }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx - 2.5, y: cy + 5.0 }, D2D_POINT_2F { x: cx - 2.5, y: cy + 7.5 }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx + 2.5, y: cy + 5.0 }, D2D_POINT_2F { x: cx + 2.5, y: cy + 7.5 }, brush, stroke_w, Some(stroke));
                }
                2 => {
                    // Clock: Circle with hour/minute hands
                    target.DrawEllipse(&D2D1_ELLIPSE { point: D2D_POINT_2F { x: cx, y: cy }, radiusX: 7.0, radiusY: 7.0 }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx, y: cy }, D2D_POINT_2F { x: cx, y: cy - 4.5 }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx, y: cy }, D2D_POINT_2F { x: cx + 3.5, y: cy }, brush, stroke_w, Some(stroke));
                }
                3 => {
                    // Timers: Stopwatch
                    target.DrawEllipse(&D2D1_ELLIPSE { point: D2D_POINT_2F { x: cx, y: cy + 1.0 }, radiusX: 6.5, radiusY: 6.5 }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx, y: cy - 5.5 }, D2D_POINT_2F { x: cx, y: cy - 8.0 }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx - 2.5, y: cy - 8.0 }, D2D_POINT_2F { x: cx + 2.5, y: cy - 8.0 }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx, y: cy + 1.0 }, D2D_POINT_2F { x: cx + 3.0, y: cy - 2.0 }, brush, stroke_w, Some(stroke));
                }
                4 => {
                    // Calendar: Calendar grid
                    let rect = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F { left: cx - 7.0, top: cy - 6.0, right: cx + 7.0, bottom: cy + 7.5 },
                        radiusX: 2.0,
                        radiusY: 2.0,
                    };
                    target.DrawRoundedRectangle(&rect, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx - 7.0, y: cy - 2.0 }, D2D_POINT_2F { x: cx + 7.0, y: cy - 2.0 }, brush, stroke_w, Some(stroke));
                    // Hooks
                    target.DrawLine(D2D_POINT_2F { x: cx - 4.0, y: cy - 8.0 }, D2D_POINT_2F { x: cx - 4.0, y: cy - 6.0 }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx + 4.0, y: cy - 8.0 }, D2D_POINT_2F { x: cx + 4.0, y: cy - 6.0 }, brush, stroke_w, Some(stroke));
                }
                5 => {
                    // Weather: Cloud with sun rays
                    let cloud = factory.CreatePathGeometry()?;
                    let sink = cloud.Open()?;
                    sink.BeginFigure(D2D_POINT_2F { x: cx - 5.0, y: cy + 5.0 }, D2D1_FIGURE_BEGIN_HOLLOW);
                    sink.AddLine(D2D_POINT_2F { x: cx + 5.0, y: cy + 5.0 });
                    sink.AddArc(&D2D1_ARC_SEGMENT {
                        point: D2D_POINT_2F { x: cx + 5.0, y: cy + 0.5 },
                        size: D2D_SIZE_F { width: 3.0, height: 3.0 },
                        rotationAngle: 0.0,
                        sweepDirection: D2D1_SWEEP_DIRECTION_COUNTER_CLOCKWISE,
                        arcSize: D2D1_ARC_SIZE_SMALL,
                    });
                    sink.AddArc(&D2D1_ARC_SEGMENT {
                        point: D2D_POINT_2F { x: cx - 2.0, y: cy - 1.0 },
                        size: D2D_SIZE_F { width: 4.5, height: 4.5 },
                        rotationAngle: 0.0,
                        sweepDirection: D2D1_SWEEP_DIRECTION_COUNTER_CLOCKWISE,
                        arcSize: D2D1_ARC_SIZE_SMALL,
                    });
                    sink.AddArc(&D2D1_ARC_SEGMENT {
                        point: D2D_POINT_2F { x: cx - 5.0, y: cy + 5.0 },
                        size: D2D_SIZE_F { width: 3.5, height: 3.5 },
                        rotationAngle: 0.0,
                        sweepDirection: D2D1_SWEEP_DIRECTION_COUNTER_CLOCKWISE,
                        arcSize: D2D1_ARC_SIZE_SMALL,
                    });
                    sink.EndFigure(D2D1_FIGURE_END_OPEN);
                    sink.Close()?;
                    target.DrawGeometry(&cloud, brush, stroke_w, Some(stroke));
                    // Sun crest
                    target.DrawLine(D2D_POINT_2F { x: cx + 1.0, y: cy - 5.0 }, D2D_POINT_2F { x: cx + 2.0, y: cy - 7.0 }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx + 4.5, y: cy - 4.0 }, D2D_POINT_2F { x: cx + 6.5, y: cy - 5.5 }, brush, stroke_w, Some(stroke));
                }
                6 => {
                    // Clipboard: Board + clip
                    let board = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F { left: cx - 6.0, top: cy - 5.0, right: cx + 6.0, bottom: cy + 7.5 },
                        radiusX: 2.0,
                        radiusY: 2.0,
                    };
                    target.DrawRoundedRectangle(&board, brush, stroke_w, Some(stroke));
                    let clip_rect = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F { left: cx - 3.0, top: cy - 7.5, right: cx + 3.0, bottom: cy - 4.5 },
                        radiusX: 1.0,
                        radiusY: 1.0,
                    };
                    target.DrawRoundedRectangle(&clip_rect, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx - 3.0, y: cy - 0.5 }, D2D_POINT_2F { x: cx + 3.0, y: cy - 0.5 }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx - 3.0, y: cy + 3.0 }, D2D_POINT_2F { x: cx + 1.5, y: cy + 3.0 }, brush, stroke_w, Some(stroke));
                }
                7 => {
                    // Notes: Sticky note with folded corner
                    let geom = factory.CreatePathGeometry()?;
                    let sink = geom.Open()?;
                    sink.BeginFigure(D2D_POINT_2F { x: cx - 6.0, y: cy - 7.0 }, D2D1_FIGURE_BEGIN_HOLLOW);
                    sink.AddLine(D2D_POINT_2F { x: cx + 2.0, y: cy - 7.0 });
                    sink.AddLine(D2D_POINT_2F { x: cx + 6.5, y: cy - 2.5 });
                    sink.AddLine(D2D_POINT_2F { x: cx + 6.5, y: cy + 7.0 });
                    sink.AddLine(D2D_POINT_2F { x: cx - 6.0, y: cy + 7.0 });
                    sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                    sink.Close()?;
                    target.DrawGeometry(&geom, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx + 2.0, y: cy - 7.0 }, D2D_POINT_2F { x: cx + 2.0, y: cy - 2.5 }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx + 2.0, y: cy - 2.5 }, D2D_POINT_2F { x: cx + 6.5, y: cy - 2.5 }, brush, stroke_w, Some(stroke));
                }
                8 => {
                    // File Shelf: Folder
                    let geom = factory.CreatePathGeometry()?;
                    let sink = geom.Open()?;
                    sink.BeginFigure(D2D_POINT_2F { x: cx - 7.0, y: cy + 6.0 }, D2D1_FIGURE_BEGIN_HOLLOW);
                    sink.AddLine(D2D_POINT_2F { x: cx + 7.0, y: cy + 6.0 });
                    sink.AddLine(D2D_POINT_2F { x: cx + 7.0, y: cy - 3.0 });
                    sink.AddLine(D2D_POINT_2F { x: cx - 1.0, y: cy - 3.0 });
                    sink.AddLine(D2D_POINT_2F { x: cx - 3.0, y: cy - 6.0 });
                    sink.AddLine(D2D_POINT_2F { x: cx - 7.0, y: cy - 6.0 });
                    sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                    sink.Close()?;
                    target.DrawGeometry(&geom, brush, stroke_w, Some(stroke));
                }
                9 => {
                    // Drop: Upload arrow from tray
                    target.DrawLine(D2D_POINT_2F { x: cx - 7.0, y: cy + 3.0 }, D2D_POINT_2F { x: cx - 7.0, y: cy + 6.5 }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx - 7.0, y: cy + 6.5 }, D2D_POINT_2F { x: cx + 7.0, y: cy + 6.5 }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx + 7.0, y: cy + 6.5 }, D2D_POINT_2F { x: cx + 7.0, y: cy + 3.0 }, brush, stroke_w, Some(stroke));
                    // Arrow up
                    target.DrawLine(D2D_POINT_2F { x: cx, y: cy + 3.5 }, D2D_POINT_2F { x: cx, y: cy - 6.5 }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx - 3.5, y: cy - 3.0 }, D2D_POINT_2F { x: cx, y: cy - 6.5 }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx + 3.5, y: cy - 3.0 }, D2D_POINT_2F { x: cx, y: cy - 6.5 }, brush, stroke_w, Some(stroke));
                }
                10 => {
                    // Converters: Swap horizontal arrows
                    target.DrawLine(D2D_POINT_2F { x: cx - 6.0, y: cy - 3.0 }, D2D_POINT_2F { x: cx + 6.0, y: cy - 3.0 }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx + 3.0, y: cy - 5.5 }, D2D_POINT_2F { x: cx + 6.0, y: cy - 3.0 }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx + 3.0, y: cy - 0.5 }, D2D_POINT_2F { x: cx + 6.0, y: cy - 3.0 }, brush, stroke_w, Some(stroke));

                    target.DrawLine(D2D_POINT_2F { x: cx + 6.0, y: cy + 3.0 }, D2D_POINT_2F { x: cx - 6.0, y: cy + 3.0 }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx - 3.0, y: cy + 0.5 }, D2D_POINT_2F { x: cx - 6.0, y: cy + 3.0 }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx - 3.0, y: cy + 5.5 }, D2D_POINT_2F { x: cx - 6.0, y: cy + 3.0 }, brush, stroke_w, Some(stroke));
                }
                11 => {
                    // Links: Chain links
                    let link1 = factory.CreatePathGeometry()?;
                    let sink = link1.Open()?;
                    sink.BeginFigure(D2D_POINT_2F { x: cx - 4.0, y: cy - 1.0 }, D2D1_FIGURE_BEGIN_HOLLOW);
                    sink.AddLine(D2D_POINT_2F { x: cx - 6.0, y: cy + 1.0 });
                    sink.AddArc(&D2D1_ARC_SEGMENT {
                        point: D2D_POINT_2F { x: cx - 3.0, y: cy + 4.0 },
                        size: D2D_SIZE_F { width: 2.5, height: 2.5 },
                        rotationAngle: 0.0,
                        sweepDirection: D2D1_SWEEP_DIRECTION_COUNTER_CLOCKWISE,
                        arcSize: D2D1_ARC_SIZE_SMALL,
                    });
                    sink.AddLine(D2D_POINT_2F { x: cx - 1.0, y: cy + 2.0 });
                    sink.EndFigure(D2D1_FIGURE_END_OPEN);
                    sink.Close()?;
                    target.DrawGeometry(&link1, brush, stroke_w, Some(stroke));

                    let link2 = factory.CreatePathGeometry()?;
                    let sink2 = link2.Open()?;
                    sink2.BeginFigure(D2D_POINT_2F { x: cx + 1.0, y: cy - 2.0 }, D2D1_FIGURE_BEGIN_HOLLOW);
                    sink2.AddLine(D2D_POINT_2F { x: cx + 3.0, y: cy - 4.0 });
                    sink2.AddArc(&D2D1_ARC_SEGMENT {
                        point: D2D_POINT_2F { x: cx + 6.0, y: cy - 1.0 },
                        size: D2D_SIZE_F { width: 2.5, height: 2.5 },
                        rotationAngle: 0.0,
                        sweepDirection: D2D1_SWEEP_DIRECTION_COUNTER_CLOCKWISE,
                        arcSize: D2D1_ARC_SIZE_SMALL,
                    });
                    sink2.AddLine(D2D_POINT_2F { x: cx + 4.0, y: cy + 1.0 });
                    sink2.EndFigure(D2D1_FIGURE_END_OPEN);
                    sink2.Close()?;
                    target.DrawGeometry(&link2, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx - 2.5, y: cy + 2.5 }, D2D_POINT_2F { x: cx + 2.5, y: cy - 2.5 }, brush, stroke_w, Some(stroke));
                }
                12 => {
                    // Media: Music note
                    let note = factory.CreatePathGeometry()?;
                    let sink = note.Open()?;
                    sink.BeginFigure(D2D_POINT_2F { x: cx - 3.0, y: cy + 5.0 }, D2D1_FIGURE_BEGIN_HOLLOW);
                    sink.AddLine(D2D_POINT_2F { x: cx - 3.0, y: cy - 5.0 });
                    sink.AddLine(D2D_POINT_2F { x: cx + 4.0, y: cy - 7.0 });
                    sink.AddLine(D2D_POINT_2F { x: cx + 4.0, y: cy + 3.0 });
                    sink.EndFigure(D2D1_FIGURE_END_OPEN);
                    sink.Close()?;
                    target.DrawGeometry(&note, brush, stroke_w, Some(stroke));
                    target.FillEllipse(&D2D1_ELLIPSE { point: D2D_POINT_2F { x: cx - 4.5, y: cy + 5.0 }, radiusX: 2.5, radiusY: 2.0 }, brush);
                    target.FillEllipse(&D2D1_ELLIPSE { point: D2D_POINT_2F { x: cx + 2.5, y: cy + 3.0 }, radiusX: 2.5, radiusY: 2.0 }, brush);
                }
                _ => {
                    // "+" plus add button
                    target.DrawLine(D2D_POINT_2F { x: cx - 5.0, y: cy }, D2D_POINT_2F { x: cx + 5.0, y: cy }, brush, stroke_w, Some(stroke));
                    target.DrawLine(D2D_POINT_2F { x: cx, y: cy - 5.0 }, D2D_POINT_2F { x: cx, y: cy + 5.0 }, brush, stroke_w, Some(stroke));
                }
            }
        }
        Ok(())
    }

    pub fn draw_pin_icon(
        &self,
        target: &ID2D1DCRenderTarget,
        cx: f32,
        cy: f32,
        is_pinned: bool,
        brush: &ID2D1SolidColorBrush,
    ) -> Result<()> {
        unsafe {
            let stroke = &self.stroke_style;
            let stroke_w = 1.3;
            // Draw a neat pin: small head bar, body, point
            if is_pinned {
                // Tilted pin inserted
                target.DrawLine(D2D_POINT_2F { x: cx - 4.0, y: cy - 5.0 }, D2D_POINT_2F { x: cx + 4.0, y: cy - 5.0 }, brush, stroke_w, Some(stroke));
                target.DrawLine(D2D_POINT_2F { x: cx, y: cy - 5.0 }, D2D_POINT_2F { x: cx, y: cy + 2.0 }, brush, stroke_w + 0.5, Some(stroke));
                target.DrawLine(D2D_POINT_2F { x: cx - 3.0, y: cy + 2.0 }, D2D_POINT_2F { x: cx + 3.0, y: cy + 2.0 }, brush, stroke_w, Some(stroke));
                target.DrawLine(D2D_POINT_2F { x: cx, y: cy + 2.0 }, D2D_POINT_2F { x: cx, y: cy + 6.0 }, brush, stroke_w, Some(stroke));
            } else {
                // Outlined pin
                target.DrawLine(D2D_POINT_2F { x: cx - 3.5, y: cy - 5.0 }, D2D_POINT_2F { x: cx + 3.5, y: cy - 5.0 }, brush, stroke_w, Some(stroke));
                target.DrawLine(D2D_POINT_2F { x: cx - 2.5, y: cy - 5.0 }, D2D_POINT_2F { x: cx - 1.5, y: cy + 1.0 }, brush, stroke_w, Some(stroke));
                target.DrawLine(D2D_POINT_2F { x: cx + 2.5, y: cy - 5.0 }, D2D_POINT_2F { x: cx + 1.5, y: cy + 1.0 }, brush, stroke_w, Some(stroke));
                target.DrawLine(D2D_POINT_2F { x: cx - 3.5, y: cy + 1.0 }, D2D_POINT_2F { x: cx + 3.5, y: cy + 1.0 }, brush, stroke_w, Some(stroke));
                target.DrawLine(D2D_POINT_2F { x: cx, y: cy + 1.0 }, D2D_POINT_2F { x: cx, y: cy + 6.0 }, brush, stroke_w, Some(stroke));
            }
        }
        Ok(())
    }

    pub fn draw_gear_icon(
        &self,
        target: &ID2D1DCRenderTarget,
        cx: f32,
        cy: f32,
        brush: &ID2D1SolidColorBrush,
    ) -> Result<()> {
        unsafe {
            let stroke = &self.stroke_style;
            let stroke_w = 1.3;
            // Center ring
            let ellipse = D2D1_ELLIPSE {
                point: D2D_POINT_2F { x: cx, y: cy },
                radiusX: 3.2,
                radiusY: 3.2,
            };
            target.DrawEllipse(&ellipse, brush, stroke_w, Some(stroke));
            // 6 radial spokes
            for k in 0..6 {
                let angle = (k as f32) * (std::f32::consts::PI / 3.0);
                let x1 = cx + angle.cos() * 3.8;
                let y1 = cy + angle.sin() * 3.8;
                let x2 = cx + angle.cos() * 6.2;
                let y2 = cy + angle.sin() * 6.2;
                target.DrawLine(D2D_POINT_2F { x: x1, y: y1 }, D2D_POINT_2F { x: x2, y: y2 }, brush, stroke_w + 0.4, Some(stroke));
            }
        }
        Ok(())
    }

    pub fn draw_close_icon(
        &self,
        target: &ID2D1DCRenderTarget,
        cx: f32,
        cy: f32,
        brush: &ID2D1SolidColorBrush,
    ) -> Result<()> {
        unsafe {
            let stroke = &self.stroke_style;
            let stroke_w = 1.3;
            target.DrawLine(D2D_POINT_2F { x: cx - 4.0, y: cy - 4.0 }, D2D_POINT_2F { x: cx + 4.0, y: cy + 4.0 }, brush, stroke_w, Some(stroke));
            target.DrawLine(D2D_POINT_2F { x: cx + 4.0, y: cy - 4.0 }, D2D_POINT_2F { x: cx - 4.0, y: cy + 4.0 }, brush, stroke_w, Some(stroke));
        }
        Ok(())
    }

    pub fn draw_header_logo(
        &self,
        target: &ID2D1DCRenderTarget,
        factory: &ID2D1Factory,
        rect: &D2D_RECT_F,
        brush: &ID2D1SolidColorBrush,
        bg: &ID2D1SolidColorBrush,
        notch_fill: &ID2D1SolidColorBrush,
    ) -> Result<()> {
        unsafe {
            let round = D2D1_ROUNDED_RECT {
                rect: *rect,
                radiusX: 6.0,
                radiusY: 6.0,
            };
            target.FillRoundedRectangle(&round, bg);
            target.DrawRoundedRectangle(&round, brush, 1.0, Some(&self.stroke_style));

            // Small notch emblem attached to top center of logo
            let notch_w = (rect.right - rect.left) * 0.45;
            let notch_h = (rect.bottom - rect.top) * 0.35;
            let n_left = rect.left + ((rect.right - rect.left - notch_w) / 2.0);
            let n_top = rect.top;

            let geom = factory.CreatePathGeometry()?;
            let sink = geom.Open()?;
            sink.BeginFigure(D2D_POINT_2F { x: n_left, y: n_top }, D2D1_FIGURE_BEGIN_FILLED);
            sink.AddLine(D2D_POINT_2F { x: n_left + notch_w, y: n_top });
            sink.AddLine(D2D_POINT_2F { x: n_left + notch_w - 1.0, y: n_top + notch_h - 2.0 });
            sink.AddArc(&D2D1_ARC_SEGMENT {
                point: D2D_POINT_2F { x: n_left + notch_w - 3.0, y: n_top + notch_h },
                size: D2D_SIZE_F { width: 2.0, height: 2.0 },
                rotationAngle: 0.0,
                sweepDirection: D2D1_SWEEP_DIRECTION_CLOCKWISE,
                arcSize: D2D1_ARC_SIZE_SMALL,
            });
            sink.AddLine(D2D_POINT_2F { x: n_left + 3.0, y: n_top + notch_h });
            sink.AddArc(&D2D1_ARC_SEGMENT {
                point: D2D_POINT_2F { x: n_left + 1.0, y: n_top + notch_h - 2.0 },
                size: D2D_SIZE_F { width: 2.0, height: 2.0 },
                rotationAngle: 0.0,
                sweepDirection: D2D1_SWEEP_DIRECTION_CLOCKWISE,
                arcSize: D2D1_ARC_SIZE_SMALL,
            });
            sink.EndFigure(D2D1_FIGURE_END_CLOSED);
            sink.Close()?;
            target.FillGeometry(&geom, notch_fill, None);
        }
        Ok(())
    }
}
