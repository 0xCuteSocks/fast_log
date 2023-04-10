#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;
use rtrb::RingBuffer;
use std::thread;
use time::format_description;

struct RawFunc {
    data: Box<dyn Fn() + Send + 'static>,
}

impl RawFunc {
    fn new<T>(data: T) -> Self
    where
        T: Fn() + Send + 'static,
    {
        RawFunc {
            data: Box::new(data),
        }
    }

    fn invoke(self) {
        (self.data)()
    }
}

fn main() {
    let (mut tx, mut rx) = RingBuffer::<(std::time::Instant, RawFunc)>::new(8);

    let t1 = thread::spawn(move || {
        let core_ids = core_affinity::get_core_ids().unwrap();
        core_affinity::set_for_current(*core_ids.get(2).unwrap());

        let mut i = 0;
        while i < 10000 {
            match rx.pop() {
                Ok(msg) => {
                    println!("{:?}", msg.0.elapsed());
                    msg.1.invoke();
                    i += 1;
                }
                Err(_e) => {}
            }
        }
    });

    let t2 = thread::spawn(move || {
        let core_ids = core_affinity::get_core_ids().unwrap();
        core_affinity::set_for_current(*core_ids.get(4).unwrap());

        for _ in 0..10000 {
            let date = time::OffsetDateTime::now_utc();
            while !tx
                .push((
                    std::time::Instant::now(),
                    RawFunc::new(move || {
                        let ins = std::time::Instant::now();
                        println!(
                            "ts: {} volume: {} price: {} flag: {}",
                            date.format(&format_description::well_known::Rfc3339)
                                .unwrap(),
                            100.02,
                            20000.0,
                            true
                        );
                        println!("println! cost {:?}", ins.elapsed());
                    }),
                ))
                .is_ok()
            {}
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    });

    t1.join().expect("Couldn't join on the associated thread1");
    t2.join().expect("Couldn't join on the associated thread2");
}
