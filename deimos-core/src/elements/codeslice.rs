#[derive(Debug,Clone)]
pub struct CodeSlice {
    // for generating the section of code
    abs_start : usize,
    abs_end : usize,

    // for user display
    line_no : usize,
    start : usize,
    end : usize,
}

impl std::default::Default for CodeSlice {
    fn default() -> CodeSlice {
        CodeSlice {
            abs_start : 0,
            abs_end : 0,
            line_no : 0,
            start : 0,
            end : 0
        }
    }
}

impl CodeSlice {
    pub fn empty() -> CodeSlice { CodeSlice::default() }

    pub fn new(abs_start : usize, abs_end : usize, line : usize, line_start_pos : usize) -> CodeSlice {
        CodeSlice {
            abs_start : abs_start, abs_end : abs_end,
            line_no : line,
            start : abs_start - line_start_pos + 1,
            end : abs_end - line_start_pos + 1,
        }
    }

    pub fn create_from(slice1 : &CodeSlice, slice2 : &CodeSlice) -> CodeSlice {
        CodeSlice {
            abs_start : slice1.abs_start,
            abs_end : slice2.abs_end,
            line_no : slice1.line_no,
            start : slice1.start,
            end : slice2.end,
        }
    }    

    pub fn get_range(&self) -> (usize,usize) {
        (self.abs_start, self.abs_end)
    }

    pub fn get_line(&self) -> usize {
        self.line_no
    }

    pub fn get_column(&self) -> usize {
        self.start
    }

    pub fn slice_code<'a>(&self, raw_code : &'a str) -> &'a str {
        &raw_code[self.abs_start .. self.abs_end]
    }
}