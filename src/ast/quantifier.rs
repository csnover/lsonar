#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quantifier {
    Star,     // * (0 or more, greedy)
    Plus,     // + (1 or more, greedy)
    Question, // ? (0 or 1, greedy)
    Minus,    // - (0 or more, non-greedy/shortest)
}
