extern "C" {
    static kvma_beg: u8;
    static kvma_end: u8;

    static kwm_beg: u8;
    static kwm_end: u8;
}

fn f_kvma_start() -> usize {
    unsafe { &kvma_beg as *const _ as usize }
}

fn f_kvma_end() -> usize {
    unsafe { &kvma_end as *const _ as usize }
}

fn f_kwm_start() -> usize {
    unsafe { &kwm_beg as *const _ as usize }
}

fn f_kwm_end() -> usize {
    unsafe { &kwm_end as *const _ as usize }
}

mod link {
    use super::{f_kvma_end, f_kvma_start, f_kwm_end, f_kwm_start};
    use libmm::{links_kvma_end, links_kvma_start, links_kwm_end, links_kwm_start};

    links_kvma_start!(f_kvma_start);
    links_kvma_end!(f_kvma_end);
    links_kwm_start!(f_kwm_start);
    links_kwm_end!(f_kwm_end);
}
