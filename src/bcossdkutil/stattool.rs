use time::Tm;

pub struct StatTime{
    pub time_begin:Tm,
    pub time_end:Tm

}

impl StatTime{
    pub fn begin()-> Self{
        StatTime{
            time_begin : time::now(),
            time_end : time::now()
        }
    }
    pub fn done(&mut self){
        self.time_end = time::now();
    }
    pub fn used_ms(&self)->i64{
        let time_used = self.time_end - self.time_begin;
        return time_used.num_milliseconds()
    }
}