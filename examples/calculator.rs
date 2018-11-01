extern crate skryn;
extern crate webrender;

use std::sync::{Arc,Mutex};
use std::any::Any;
use std::thread;
use std::time::Duration;

use skryn::data::*;
use skryn::gui::font::FontStore;
use skryn::gui::properties::{Property, Extent, Properties, IdGenerator};
use skryn::elements::*;

use webrender::api::{ColorF, DisplayListBuilder, RenderApi};

#[derive(Debug, Clone)]
enum Operation {
    Input(f64),
    Add,
    Subtract,
    Multiply,
    Divide,
    Answer,
}

#[derive(Debug, Clone)]
struct Calculator{
    ops: Vec<Operation>
}

impl Calculator {
    fn new() -> Calculator{
        Calculator {
            ops: vec![],
        }
    }

    fn push_num(&mut self, n: f64) {
        let ops = self.ops.clone();
        if self.ops.is_empty() {
            self.ops.push(Operation::Input(n));
        } else if let Some(x) = ops.last() {
            if let Operation::Input(_i) = x {
                //no op
            } else {
                self.ops.push(Operation::Input(n))
            }
        }
    }

    fn push_op(&mut self, op: Operation) -> Option<f64>{
        let ops = self.ops.clone();
        let len = self.ops.len();
        let mut return_value = None;
        match op {
            Operation::Answer => {
                if len > 2 {
                    match (&ops[len - 3],&ops[len - 1]) {
                        (Operation::Input(l),Operation::Input(r)) => {
                            match &ops[len - 2] {
                                Operation::Add => {
                                    self.ops.push(Operation::Answer);
                                    return_value = Some(l+r);
                                    //self.ops.push(Operation::Input(l+r));
                                },
                                Operation::Subtract => {
                                    self.ops.push(Operation::Answer);
                                    return_value = Some(l-r);
                                    //self.ops.push(Operation::Input(l-r));
                                },
                                Operation::Multiply => {
                                    self.ops.push(Operation::Answer);
                                    return_value = Some(l*r);
                                    //self.ops.push(Operation::Input(l*r));
                                },
                                Operation::Divide => {
                                    self.ops.push(Operation::Answer);
                                    return_value = Some(l/r);
                                    //self.ops.push(Operation::Input(l/r));
                                },
                                _ => ()
                            }
                        },
                        _ => ()
                    }
                }
            },
            Operation::Input(n) => {
                self.push_num(n);
            }
            _ => {
                if let Some(xop)  = ops.last() {
                    match xop {
                        Operation::Input(_) => {
                            self.ops.push(op);
                        },
                        _ => ()
                    }
                }
            }
        }

        return_value
    }
}

#[derive(Clone)]
struct CalculatorView{
    calc: Arc<Mutex<Calculator>>,
    view: Arc<Mutex<VBox>>,
    textbox: Arc<Mutex<TextBox>>,
}

impl CalculatorView{
    fn new() -> CalculatorView{
        let calc = Arc::new(Mutex::new(Calculator::new()));
        let mut view = VBox::new();
        let mut tbox = TextBox::new("".to_owned());
        tbox.set_singleline(true);
        tbox.set(skryn::gui::properties::Property::Height(skryn::gui::properties::Unit::Stretch(1.0)));
        tbox.set(skryn::gui::properties::Property::Size(32));
        let tbox = Arc::new(Mutex::new(tbox));
        view.append(tbox.clone());

        let mut hbox = HBox::new();
        let mut addbutt = Button::new("+".to_owned());
        let tmpbox = tbox.clone();
        let tmpcalc = calc.clone();
        addbutt.set_handler(skryn::elements::ElementEvent::Clicked, move |_, _|{
            let tb = tmpbox.lock().unwrap();
            /*let val = tb.get_value().parse::<f64>();
            if let Ok(n) = val {
                tmpcalc.lock().unwrap().push_num(n);
                tmpcalc.lock().unwrap().push_op(Operation::Add);
            } else {*/
                Alert::show(format!("Value {} could not be parsed to a number.",1/*tb.get_value()*/), "Number Error".to_owned());
            //}

            true
        });
        hbox.append(Arc::new(Mutex::new(addbutt)));
        let mut subbutt = Button::new("-".to_owned());
        hbox.append(Arc::new(Mutex::new(subbutt)));
        let mut mulbutt = Button::new("*".to_owned());
        hbox.append(Arc::new(Mutex::new(mulbutt)));
        let mut divbutt = Button::new("/".to_owned());
        hbox.append(Arc::new(Mutex::new(divbutt)));
        let mut eqlbutt = Button::new("=".to_owned());
        hbox.append(Arc::new(Mutex::new(eqlbutt)));

        view.append(Arc::new(Mutex::new(hbox)));

        CalculatorView{
            calc: calc,
            view: Arc::new(Mutex::new(view)),
            textbox: tbox,
        }
    }


}

struct Alert;
impl Alert{
    fn show(message: String, heading: String){
        let msg_box = TextBox::new(message);
        skryn::gui::window::Manager::add(Arc::new(Mutex::new(msg_box)),heading,400.0,100.0);
    }
}

fn main(){
    let mut calc = CalculatorView::new();

    //Calc.push_num(1.0);
    /*calc.push_op(Operation::Input(1.0));
    calc.push_op(Operation::Add);
    calc.push_op(Operation::Input(1.0));
    calc.push_op(Operation::Answer);
    calc.push_op(Operation::Add);
    calc.push_op(Operation::Input(1.0));
    calc.push_op(Operation::Answer);
    calc.push_op(Operation::Input(1.0)); // should be ignored?*/


    //println!("{:?}", calc.ops);

    skryn::gui::window::Manager::add(calc.view.clone(),String::from("Calculator"), 300.0, 200.0);
    skryn::gui::window::Manager::start(60);
}