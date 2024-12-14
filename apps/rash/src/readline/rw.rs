use super::*;

impl Readline {
    pub(super) fn read_char(&self) -> char {
        fn get_utf8_char_size(byte: u8) -> usize {
            let mut len = 0;
            let mut mask = 0x80u8;
            while byte & mask != 0 {
                len += 1;
                if len > 6 {
                    unreachable!();
                }
                mask >>= 1;
            }
            if len == 0 {
                return 1;
            }
            len
        }

        let mut buf = [0; 1];
        raca_std::io::stdin().read_exact(&mut buf);

        let size = get_utf8_char_size(buf[0]);
        let mut char = buf[0] as u32;
        if size == 0 {
            return '\0';
        }
        for _ in 1..size {
            let mut tmp_buf = [0; 1];
            raca_std::io::stdin().read_exact(&mut tmp_buf);
            char <<= 8;
            char |= tmp_buf[0] as u32;
        }
        char::from_u32(char).unwrap()
    }

    pub(super) fn show_char(&mut self, key: char, buf: &mut String) {
        print!("{}", key);
        let idx = self.insert_point;

        for index in idx..buf.len() {
            print!("{}", buf.chars().nth(index).unwrap());
        }
        for _ in idx..buf.len() {
            print!("\x08");
        }
        buf.insert(idx, key);
        self.insert_point += 1;
    }
}
