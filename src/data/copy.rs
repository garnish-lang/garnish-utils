use garnish_lang_traits::GarnishData;

pub fn copy_over_data<Data: GarnishData>(from: &Data, to: &Data) -> Result<(), Data::Error> {
    todo!()
}

#[cfg(test)]
mod tests {
    use garnish_lang_simple_data::SimpleGarnishData;

    #[test]
    fn copy_number() {
        let from = SimpleGarnishData::new();
        let to = SimpleGarnishData::new();
    }
}