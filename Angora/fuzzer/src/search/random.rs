use super::*;

use std::sync::RwLock;

pub struct RandomSearch<'a> {
    handler: SearchHandler<'a>,
    enable_dict: bool,
    //dictionary: Dict,
}

impl<'a> RandomSearch<'a> {
    pub fn new(handler: SearchHandler<'a>, enable_dict: bool) -> Self {
        Self { handler, enable_dict }
    }

    pub fn run(&mut self) {
        let mut input = self.handler.get_f_input();
        let orig_input_val = input.get_value();

        /*{
            let cmpid = self.handler.cond.base.cmpid;
            let offsets = self.handler.cond.offsets.clone();
            let offsets_opt = self.handler.cond.offsets_opt.clone();
            let mut d = match self.handler.executor.dictionary.read() {
                Ok(guard) => guard,
                Err(poisoned) => {
                    warn!("Lock poisoned. Results can be incorrect! Continuing...");
                    poisoned.into_inner()
                }
            };
            match d.0.contains_key(&cmpid) {
                true => {
                    info!("{:?} cmpid: {}, offsets: {:?}, offsets_opt: {:?}", d.0.get(&cmpid).unwrap(), cmpid, offsets, offsets_opt);
                }
                false => {
                    info!("not exists {}", cmpid);
                }
            }
        }*/

        loop {
            if self.handler.is_stopped_or_skip() {
                break;
            }
            input.assign(&orig_input_val);
            input.randomize_all(self.enable_dict, self.handler.executor.dictionary.clone());
            let ret = self.handler.execute_cond(&input).1;
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
        }
    }
}
