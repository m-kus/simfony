use simplicity::bit_machine::ExecTracker;
use simplicity::ffi::ffi::UWORD;
use simplicity::jet::Jet;

pub struct Tracker;

impl ExecTracker for Tracker {
    fn track_left(&mut self, _: simplicity::Ihr) {}
    fn track_right(&mut self, _: simplicity::Ihr) {}
    fn track_jet_call<J: Jet>(
        &mut self,
        jet: &J,
        input_buffer: &[UWORD],
        output_buffer: &[UWORD],
        success: bool,
    ) {
        println!("{:?} {:?} {:?} {:?}", jet, input_buffer, output_buffer, success);
    }
}
