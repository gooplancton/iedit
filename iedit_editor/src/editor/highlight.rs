pub type SelectionRage = ((usize, usize), (usize, usize));
pub enum RangeHighlight {
    None,
    WholeLine,
    Before(usize),
    After(usize),
    Range(usize, usize),
}

impl RangeHighlight {
    pub fn new(line_idx: usize, selection_range: &SelectionRage) -> Self {
        let ((start_x, start_y), (end_x, end_y)) = *selection_range;
        if start_y == end_y && start_y == line_idx {
            RangeHighlight::Range(start_x, end_x)
        } else if start_y == line_idx {
            RangeHighlight::After(start_x)
        } else if end_y == line_idx {
            RangeHighlight::Before(end_x)
        } else if start_y < line_idx && end_y > line_idx {
            RangeHighlight::WholeLine
        } else {
            RangeHighlight::None
        }
    }
}
