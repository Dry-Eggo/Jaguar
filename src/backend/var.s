
    fn is_generic_context(&mut self) -> bool {
        let mut depth = 1;
        let mut i = 1;
        while let Some(token) = self.get(i) {
            println!("Here");
            match token.kind {
                TokenType::Operator(val) if val == ">" => {
                    depth -= 1;
                    if depth == 0 {
                        let next = self.get(i + 1);
                        match next.unwrap().kind {
                            TokenType::Separator(v) if v == "(" => return true,
                            TokenType::DOT | TokenType::DCOLON => return true,
                            TokenType::Operator(v) if v == "<" => depth += 1,
                            TokenType::Operator(v) if v == "=" => return false,
                            TokenType::Separator(v) if v == ";" || v == "," => return false,
                            _ => {}
                        }
                        i += 1;
                    }
                }
                TokenType::Operator(val) if val == "<" => depth += 1,
                _ => {}
            }
        }
        false
    }
