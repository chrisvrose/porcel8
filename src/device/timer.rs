use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle, sleep};
use std::time::Duration;

pub struct Timer{
    timer_left: Arc<Mutex<u16>>,
    join_handle: Option<(JoinHandle<()>,std::sync::mpsc::Sender<()>)>
}

impl Timer{
    pub fn new()->Timer{
        Timer{timer_left:Arc::new(Mutex::default()),join_handle:None}
    }
    pub fn start(&mut self){
        let timer_left_ref = self.timer_left.clone();
        let (sender,receiver) = std::sync::mpsc::channel();
        let res = std::thread::spawn(move ||{
            loop{
                let val = receiver.try_recv();
                if let Ok(()) = val{
                    break;
                }else if let Err(std::sync::mpsc::TryRecvError::Disconnected) = val{
                    panic!("Disconnected");
                }
                {
                    let mut timer_lock = timer_left_ref.lock().expect("Failed to lock");
                    if *timer_lock >0 {
                        *timer_lock -= 1;
                    }
                }
                sleep(Duration::from_secs_f32(1f32/60f32));
            }

        });
        self.join_handle = Some((res,sender));
    }
    /// Set a timer down tick from `val`
    pub fn set_timer(& self,val:u16){
        let mut timer_val = self.timer_left.lock().expect("Failed to get mutex");
        *timer_val = val;
    }

    pub fn poll_value(&self)->u16{
        let res = self.timer_left.lock().expect("Failed to lock?");
        res.clone()
    }

    pub fn stop(self){
        if let Some((u,x)) = self.join_handle{
            x.send(()).expect("Failed to send signal to close thread");
            u.join().expect("Failed to close thread");
        }else{
            log::warn!("Nothing present!");
        }
    }
    pub fn send_stop_signal(&mut self){
        if let Some((_,x)) = &self.join_handle{
            x.send(()).expect("Failed to send signal to close thread");
        }else{
            log::warn!("Nothing present!");
        }
    }

}
