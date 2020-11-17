use ogmarkup::generator::Output;
use ogmarkup::typography::Space;

#[derive(Serialize)]
pub struct Html(String);

impl Html {
    fn push_str(&mut self, s : &str) -> () {
        self.0.push_str(s);
    }

    pub fn to_string(self) -> String {
        self.0
    }
}

impl Output for Html {
    fn empty(input_size : usize) -> Html {
        Html(String::with_capacity((15 * input_size) / 10))
    }

    fn render_space(&mut self, space : Space) -> () {
        self.push_str(match space {
            Space::Normal => " ",
            Space::Nbsp => "&nbsp;",
            Space::None => "",
        })
    }

    fn render_word(&mut self, word : &str) -> () {
        self.push_str(word)
    }

    fn render_mark(&mut self, mark : &str) -> () {
        self.push_str(mark)
    }

    fn render_illformed(&mut self, err : &str) -> () {
        self.push_str(err)
    }

    fn emph_template<F>(&mut self, format : F) -> ()
    where
        F : FnOnce(&mut Html) -> (),
    {
        self.push_str("<em>");
        format(self);
        self.push_str("</em>");
    }

    fn strong_emph_template<F>(&mut self, format : F) -> ()
    where
        F : FnOnce(&mut Html) -> (),
    {
        self.push_str("<strong>");
        format(self);
        self.push_str("</strong>");
    }

    fn reply_template<F>(&mut self, reply : F, _author : &Option<&str>) -> ()
    where
        F : FnOnce(&mut Html) -> (),
    {
        self.push_str("<span class=\"reply\">");
        reply(self);
        self.push_str("</span>");
    }

    fn thought_template<F>(&mut self, reply : F, author : &Option<&str>) -> ()
    where
        F : FnOnce(&mut Html) -> (),
    {
        self.push_str("<span class=\"thought");
        author.map(|a| {
            self.push_str(" by-");
            self.push_str(a);
        });
        self.push_str("\">");
        reply(self);
        self.push_str("</span>");
    }

    fn dialogue_template<F>(&mut self, reply : F, author : &Option<&str>) -> ()
    where
        F : FnOnce(&mut Html) -> (),
    {
        self.push_str("<span class=\"dialogue");
        author.map(|a| {
            self.push_str(" by-");
            self.push_str(a);
        });
        self.push_str("\">");
        reply(self);
        self.push_str("</span>");
    }

    fn between_dialogue(&mut self) -> () {
        self.push_str("</p><p>");
    }

    fn illformed_inline_template<F>(&mut self, err : F) -> ()
    where
        F : FnOnce(&mut Html) -> (),
    {
        self.push_str("<span class=\"illformed_inline\">");
        err(self);
        self.push_str("</span>");
    }

    fn paragraph_template<F>(&mut self, para : F) -> ()
    where
        F : FnOnce(&mut Html) -> (),
    {
        self.push_str("<p>");
        para(self);
        self.push_str("</p>");
    }

    fn illformed_block_template<F>(&mut self, err : F) -> ()
    where
        F : FnOnce(&mut Html) -> (),
    {
        self.push_str("<div class=\"illformed_block\">");
        err(self);
        self.push_str("</div>");
    }

    fn story_template<F>(&mut self, story : F) -> ()
    where
        F : FnOnce(&mut Html) -> (),
    {
        self.push_str("<div class=\"story\">");
        story(self);
        self.push_str("</div>");
    }

    fn aside_template<F>(&mut self, cls : &Option<&str>, aside : F) -> ()
    where
        F : FnOnce(&mut Html) -> (),
    {
        self.push_str("<div class=\"aside");
        cls.map(|c| {
            self.push_str(" ");
            self.push_str(c);
        });
        self.push_str("\">");
        aside(self);
        self.push_str("</div>");
    }
}
