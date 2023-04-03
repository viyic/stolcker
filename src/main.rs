use std::fmt;
use std::env;
use std::error::Error;
use std::collections::HashMap;
use druid::widget::{Button, Flex, Label, Painter, TextBox, Either};
use druid::{
    AppLauncher, LocalizedString, Widget, WidgetExt, WindowDesc, Data,
    theme, RenderContext, Lens, Color
};
use serde::Deserialize;
use serde_json;
use reqwest;

#[derive(Deserialize, Debug, Clone, Data)]
struct Price {
    open: f32,
    high: f32,
    low: f32,
    close: f32,
    volume: f32,
}

#[derive(Debug, Clone, Data)]
struct Record {
    time: String,
    price: Price,
}

#[derive(Debug, Clone, Data)]
struct Stock {
    #[data(ignore)]
    records: Vec<Record>,
    highest: f32,
    lowest: f32,
    time: String,
    interval: String,
    time_zone: String,
}

#[derive(Debug)]
enum AppError {
    ArgsError,
    ParseError,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AppError {}", self)
    }
}

impl Error for AppError {}

#[derive(Clone, Data, Lens)]
struct AppState {
    stock_name: String,
    stock_name_prev: String,
    stock_name_input: bool,

    stock: Stock,
    valid: bool,
    // timer_id: Timer,
    api_key: String,
}

const MAX_REQUEST_PER_DAY: i32 = 500;
const INTERVAL: &str = "15min";

// #[tokio::main]
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("Usage: {} ALPHA_VANTAGE_API_KEY", args[0]);
        return Ok(());
        // Err(AppError::ArgsError)?
    }
    let api_key = args[1].clone();

    let window_main = WindowDesc::new(ui_builder())
        .title("Stolcker");
    let stock_name_default = "GOOGL".to_string();
    let response = get_data(&stock_name_default, &api_key);
    let stock = parse_response(response)?;

    let app = AppState {
        stock: stock,
        stock_name: stock_name_default.clone(),
        stock_name_prev: stock_name_default.clone(),
        stock_name_input: true,
        valid: true,
        api_key: api_key,
    };

    // @todo: return AppLauncher PlatformError
    AppLauncher::with_window(window_main)
        .log_to_console()
        .launch(app);
    Ok(())

    // println!("{}", response);

    // Ok(())
    // let test = Test;
    // println!("{:?}", test);
    // let val: serde_json::Value = serde_json::from_str(json).unwrap();
    // let comp = Complex { real: 3.3, imag: 7.2 };
    // println!("Display:\t{}", comp);
    // println!("Debug:\t\t{:?}", comp);
}

fn get_data(stock_name: &String, api_key: &String) -> String {
    reqwest::blocking::get(format!("https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY&symbol={stock_name}&interval={INTERVAL}&apikey={api_key}")).unwrap()
        .text().unwrap()
}

fn parse_response(response: String) -> Result<Stock, Box<dyn Error>> {
    let val: serde_json::Value = serde_json::from_str(&response)?;
    let mut result = Stock {
        records: Vec::with_capacity(100),
        highest: 0.0, lowest: 0.0,
        time: val["Meta Data"]["3. Last Refreshed"].as_str().ok_or(AppError::ParseError)?.to_string(),
        interval: val["Meta Data"]["4. Interval"].as_str().ok_or(AppError::ParseError)?.to_string(),
        time_zone: val["Meta Data"]["6. Time Zone"].as_str().ok_or(AppError::ParseError)?.to_string(),
    };
    let records_map_value: serde_json::Value = serde_json::from_value(val[format!("Time Series ({INTERVAL})")].clone())?;
    let records_keys: Vec<_> = records_map_value.as_object().ok_or(AppError::ParseError)?.keys().collect();
    let records_values: Vec<_> = records_map_value.as_object().ok_or(AppError::ParseError)?.values().collect();
    let mut init = true;
    for i in 0..records_values.len() {
        let price = Price {
            open: records_values[i]["1. open"].as_str().ok_or(AppError::ParseError)?.parse()?,
            high: records_values[i]["2. high"].as_str().ok_or(AppError::ParseError)?.parse()?,
            low: records_values[i]["3. low"].as_str().ok_or(AppError::ParseError)?.parse()?,
            close: records_values[i]["4. close"].as_str().ok_or(AppError::ParseError)?.parse()?,
            volume: records_values[i]["5. volume"].as_str().ok_or(AppError::ParseError)?.parse()?,
        };
        if init {
            result.highest = price.high;
            result.lowest = price.low;
            init = false;
        }
        if price.high > result.highest {
            result.highest = price.high;
        }
        if price.low < result.lowest {
            result.lowest = price.low;
        }
        result.records.push(Record { time: records_keys[i].clone(), price: price });
    }
    // println!("{:#?}", records_keys);
    // println!("{:#?}", records_values);

    // println!("{}", val["Meta Data"]["2. Symbol"].as_str()?);
    // println!("{:?}", records.get("2023-03-24 20:00:00"));
    Ok(result)
}

fn button_blue<T> (text: impl Into<druid::widget::LabelText<T>>) -> druid::widget::Container<T> where T: druid::Data {
    Label::<T>::new(text)
        .with_text_size(24.0)
        .padding(5.0)
        .center()
        .background(Painter::new(|ctx, _, env| {
            let bounds = ctx.size().to_rect();

            ctx.fill(bounds, &env.get(theme::PRIMARY_DARK));

            if ctx.is_hot() {
                ctx.stroke(bounds.inset(-0.5), &Color::WHITE, 1.0);
                println!("{:?}", bounds);
            }
            if ctx.is_active() {
                ctx.fill(bounds, &env.get(theme::PRIMARY_LIGHT));
            }
        }))
}

fn button_check() -> impl Widget<AppState> {
    Label::new("")
        .with_text_size(24.0)
        .padding(5.0)
        .center()
        .background(Painter::new(|ctx, _, env| {
            let bounds = ctx.size().to_rect();

            ctx.fill(bounds, &Color::rgb8(0x11, 0xee, 0x11));

            if ctx.is_hot() {
                ctx.stroke(bounds.inset(-0.5), &Color::WHITE, 1.0);
                println!("{:?}", bounds);
            }
            if ctx.is_active() {
                ctx.fill(bounds, &Color::rgb8(0x11, 0xff, 0x11));
            }

            let stroke_size = 5.0;
            let style = druid::piet::StrokeStyle::new()
                .line_cap(druid::piet::LineCap::Square);
            {
                let p0 = druid::Point::new(bounds.x1 / 4.0, bounds.y1 / 2.0);
                let p1 = druid::Point::new(bounds.x1 / 2.0, bounds.y1 / 3.0 * 2.0);
                ctx.stroke_styled(druid::piet::kurbo::Line::new(p0, p1), &Color::WHITE, stroke_size, &style);
            }
            {
                let p0 = druid::Point::new(bounds.x1 / 2.0, bounds.y1 / 3.0 * 2.0);
                let p1 = druid::Point::new(bounds.x1 / 4.0 * 3.0, bounds.y1 / 3.0);
                ctx.stroke_styled(druid::piet::kurbo::Line::new(p0, p1), &Color::WHITE, stroke_size, &style);
            }
        }))
}

fn button_x () -> impl Widget<AppState> {
    Label::new("")
        .with_text_size(24.0)
        .padding(5.0)
        .center()
        .background(Painter::new(|ctx, _, env| {
            let bounds = ctx.size().to_rect();

            ctx.fill(bounds, &Color::rgb8(0xee, 0x11, 0x11));

            if ctx.is_hot() {
                ctx.stroke(bounds.inset(-0.5), &Color::WHITE, 1.0);
                println!("{:?}", bounds);
            }
            if ctx.is_active() {
                ctx.fill(bounds, &Color::rgb8(0xff, 0x11, 0x11));
            }

            let stroke_size = 5.0;
            let style = druid::piet::StrokeStyle::new()
                .line_cap(druid::piet::LineCap::Square);
            {
                let p0 = druid::Point::new(bounds.x1 / 4.0, bounds.y1 / 4.0);
                let p1 = druid::Point::new(bounds.x1 / 4.0 * 3.0, bounds.y1 / 4.0 * 3.0);
                ctx.stroke_styled(druid::piet::kurbo::Line::new(p0, p1), &Color::WHITE, stroke_size, &style);
            }
            {
                let p0 = druid::Point::new(bounds.x1 / 4.0, bounds.y1 / 4.0 * 3.0);
                let p1 = druid::Point::new(bounds.x1 / 4.0 * 3.0, bounds.y1 / 4.0);
                ctx.stroke_styled(druid::piet::kurbo::Line::new(p0, p1), &Color::WHITE, stroke_size, &style);
            }
        }))
}

fn ui_builder() -> impl Widget<AppState> {
    let label = Label::new(|data: &AppState, _env: &druid::Env| format!("{} {} ({})", data.stock.time, data.stock.time_zone, data.stock.interval))
        .padding(5.0)
        .align_right();
    let lbl_error = Label::new(|data: &AppState, _env: &druid::Env| "Error".to_string())
        .padding(5.0);
    let btn_go = button_check()
        .on_click(|_, data: &mut AppState, _| {
            if !data.stock_name.is_empty() {
                if data.stock_name != data.stock_name_prev {
                    let result = parse_response(get_data(&data.stock_name, &data.api_key));
                    match result {
                        Ok(val) => {
                            data.stock = val;
                            data.valid = true;
                        },
                        Err(err) => {
                            println!("Oops!");
                            data.valid = false;
                        },
                    }
                    // data.stock.update = true;
                }
            } else {
                data.stock_name = data.stock_name_prev.clone();
            }
            data.stock_name_input = !data.stock_name_input;
        });
    let btn_cancel = button_x()
        .on_click(|_, data: &mut AppState, _| {
            if data.stock_name != data.stock_name_prev {
                data.stock_name = data.stock_name_prev.clone();
            }
            data.stock_name_input = !data.stock_name_input;
        });
    let btn_change_stock_name = button_blue(|data: &AppState, _env: &druid::Env| data.stock_name.clone())
        .on_click(|_, data: &mut AppState, _| {
            if data.stock_name_prev != data.stock_name {
                data.stock_name_prev = data.stock_name.clone();
            }
            data.stock_name_input = !data.stock_name_input;
        });
    let text_box_stock_name = TextBox::new()
        .with_placeholder("Stock Name")
        .with_text_alignment(druid::TextAlignment::Center)
        .expand_height()
        .padding(5.0)
        .lens(AppState::stock_name);

    Flex::column()
        // .must_fill_main_axis(true)
        // .cross_axis_alignment(druid::widget::CrossAxisAlignment::End)
        .with_child(
            Either::new(
                |data, _| data.stock_name_input,
                Flex::row()
                .with_flex_spacer(1.0)
                .with_child(btn_change_stock_name)
                .with_flex_child(label, 1.0)
                ,
                Flex::row()
                .must_fill_main_axis(true)
                .main_axis_alignment(druid::widget::MainAxisAlignment::Center)
                .with_flex_child(btn_cancel.fix_width(50.0), 1.0)
                .with_child(text_box_stock_name)
                .with_flex_child(btn_go.fix_width(50.0), 1.0)
                )
            .fix_height(50.0)
            )
        .with_flex_child(
            Flex::row()
            .with_flex_child(
                Either::new(
                    |data, _| data.valid,
                    Painter::new(|ctx, data: &AppState, env| {
                        let bounds = ctx.size().to_rect();
                        ctx.fill(bounds, &env.get(theme::BACKGROUND_LIGHT));

                        let padding_y = 10.0;
                        let height = bounds.y1 - padding_y * 2.0;
                        let stock_height = (data.stock.highest - data.stock.lowest) as f64;
                        let len = data.stock.records.len();
                        for i in 0..len {
                            let gap = bounds.x1 / len as f64;
                            let x = (len - (i + 1)) as f64 * gap + gap / 2.0;
                            {
                                let x0 = x - 3.0;
                                let x1 = x + 3.0;
                                let mut y0 = (1.0 - (data.stock.records[i].price.open - data.stock.lowest) as f64 / stock_height) * height - 3.0;
                                let mut y1 = (1.0 - (data.stock.records[i].price.close - data.stock.lowest) as f64 / stock_height) * height + 3.0;
                                let mut color = Color::rgb8(0x11, 0xee, 0x11);
                                if y1 > y0 {
                                    color = Color::rgb8(0xee, 0x11, 0x11);
                                    let temp = y1;
                                    y1 = y0;
                                    y0 = temp;
                                }
                                ctx.fill(druid::Rect::new(x0, y0 + padding_y, x1, y1 + padding_y), &color);
                            }
                            {
                                let x0 = x - 1.0;
                                let x1 = x + 1.0;
                                let y0 = (1.0 - (data.stock.records[i].price.high - data.stock.lowest) as f64 / stock_height) * height - 3.0;
                                let y1 = (1.0 - (data.stock.records[i].price.low - data.stock.lowest) as f64 / stock_height) * height + 3.0;
                                ctx.fill(druid::Rect::new(x0, y0 + padding_y, x1, y1 + padding_y), &Color::rgb8(0, 0, 0));
                            }
                            // ctx.draw_text(data.stock.records[i].time.clone(), druid::Point { x, y: bounds.y1 });
                        }
                    }),
                    lbl_error
                )
            , 1.0)
        , 1.0)
}
