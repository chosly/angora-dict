// Climb Hill.
use super::*;

pub struct CbhSearch<'a> {
    handler: SearchHandler<'a>,
    enable_dict: bool,
}

impl<'a> CbhSearch<'a> {
    pub fn new(handler: SearchHandler<'a>, enable_dict: bool) -> Self {
        Self { handler, enable_dict }
    }

    pub fn run(&mut self) {
        let mut input = self.handler.get_f_input();
        assert!(
            input.len() > 0,
            "Input length < 0!! {:?}",
            self.handler.cond
        );
        let mut fmin = self.handler.execute_cond(&input).0;
        let mut input_min = input.get_value();

        if input.val_len() == self.handler.cond.variables.len() {
            input.assign(&self.handler.cond.variables);
            let f = self.handler.execute_cond(&input).0;
            if f < fmin {
                fmin = f;
                input_min = input.get_value();
            }
        }

        loop {
            if self.handler.is_stopped_or_skip() {
                break;
            }
            input.assign(&input_min);
            input.randomize_all(self.enable_dict, self.handler.executor.dictionary.clone());
            let (f0, ret) = self.handler.execute_cond(&input);

            if self.enable_dict && ret.len() > 0 {
                let mut d = match self.handler.executor.dictionary.write() {
                    Ok(guard) => guard,
                    Err(poisoned) => {
                        warn!("Lock poisoned. Results can be incorrect! Continuing...");
                        poisoned.into_inner()
                    }
                };
                d.filter(ret, self.handler.buf.clone());
            }

            if f0 < fmin {
                fmin = f0;
                input_min = input.get_value();
            }
        }

        self.handler.cond.variables = input_min;
    }
}
