/// Greek letter name to symbol mappings
/// Maps English names to their Greek Unicode equivalents

/// Convert a Greek letter name to its symbol
pub fn name_to_symbol(name: &str) -> Option<char> {
    // Check exact match first (for capital letters)
    match name {
        "Alpha" | "GAMMA" | "Gamma" => Some('Γ'),
        "Beta" => Some('Β'),
        "Delta" => Some('Δ'),
        "Epsilon" => Some('Ε'),
        "Zeta" => Some('Ζ'),
        "Eta" => Some('Η'),
        "Theta" => Some('Θ'),
        "Iota" => Some('Ι'),
        "Kappa" => Some('Κ'),
        "Lambda" => Some('Λ'),
        "Mu" => Some('Μ'),
        "Nu" => Some('Ν'),
        "Xi" => Some('Ξ'),
        "Omicron" => Some('Ο'),
        "Pi" => Some('Π'),
        "Rho" => Some('Ρ'),
        "Sigma" => Some('Σ'),
        "Tau" => Some('Τ'),
        "Upsilon" => Some('Υ'),
        "Phi" => Some('Φ'),
        "Chi" => Some('Χ'),
        "Psi" => Some('Ψ'),
        "Omega" => Some('Ω'),
        _ => {
            // Check lowercase names
            match name.to_lowercase().as_str() {
                "alpha" => Some('α'),
                "beta" => Some('β'),
                "gamma" => Some('γ'),
                "delta" => Some('δ'),
                "epsilon" => Some('ε'),
                "zeta" => Some('ζ'),
                "eta" => Some('η'),
                "theta" => Some('θ'),
                "iota" => Some('ι'),
                "kappa" => Some('κ'),
                "lambda" => Some('λ'),
                "mu" => Some('μ'),
                "nu" => Some('ν'),
                "xi" => Some('ξ'),
                "omicron" => Some('ο'),
                "pi" => Some('π'),
                "rho" => Some('ρ'),
                "sigma" => Some('σ'),
                "tau" => Some('τ'),
                "upsilon" => Some('υ'),
                "phi" => Some('φ'),
                "chi" => Some('χ'),
                "psi" => Some('ψ'),
                "omega" => Some('ω'),
                _ => None,
            }
        }
    }
}

/// Convert a Greek symbol to its English name
pub fn symbol_to_name(symbol: char) -> Option<&'static str> {
    match symbol {
        'α' => Some("alpha"),
        'β' => Some("beta"),
        'γ' => Some("gamma"),
        'δ' => Some("delta"),
        'ε' => Some("epsilon"),
        'ζ' => Some("zeta"),
        'η' => Some("eta"),
        'θ' => Some("theta"),
        'ι' => Some("iota"),
        'κ' => Some("kappa"),
        'λ' => Some("lambda"),
        'μ' => Some("mu"),
        'ν' => Some("nu"),
        'ξ' => Some("xi"),
        'ο' => Some("omicron"),
        'π' => Some("pi"),
        'ρ' => Some("rho"),
        'σ' => Some("sigma"),
        'τ' => Some("tau"),
        'υ' => Some("upsilon"),
        'φ' => Some("phi"),
        'χ' => Some("chi"),
        'ψ' => Some("psi"),
        'ω' => Some("omega"),
        'Α' => Some("Alpha"),
        'Β' => Some("Beta"),
        'Γ' => Some("Gamma"),
        'Δ' => Some("Delta"),
        'Ε' => Some("Epsilon"),
        'Ζ' => Some("Zeta"),
        'Η' => Some("Eta"),
        'Θ' => Some("Theta"),
        'Ι' => Some("Iota"),
        'Κ' => Some("Kappa"),
        'Λ' => Some("Lambda"),
        'Μ' => Some("Mu"),
        'Ν' => Some("Nu"),
        'Ξ' => Some("Xi"),
        'Ο' => Some("Omicron"),
        'Π' => Some("Pi"),
        'Ρ' => Some("Rho"),
        'Σ' => Some("Sigma"),
        'Τ' => Some("Tau"),
        'Υ' => Some("Upsilon"),
        'Φ' => Some("Phi"),
        'Χ' => Some("Chi"),
        'Ψ' => Some("Psi"),
        'Ω' => Some("Omega"),
        _ => None,
    }
}

/// Check if a string is a Greek letter name
pub fn is_greek_name(name: &str) -> bool {
    name_to_symbol(name).is_some()
}

/// Replace all Greek letter names in a string with their symbols
pub fn replace_greek_names(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch.is_alphabetic() {
            let mut word = String::new();
            word.push(ch);
            
            // Collect the full word
            while let Some(&next_ch) = chars.peek() {
                if next_ch.is_alphanumeric() || next_ch == '_' {
                    word.push(next_ch);
                    chars.next();
                } else {
                    break;
                }
            }
            
            // Try to convert to Greek symbol
            if let Some(symbol) = name_to_symbol(&word) {
                result.push(symbol);
            } else {
                result.push_str(&word);
            }
        } else {
            result.push(ch);
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_to_symbol() {
        assert_eq!(name_to_symbol("alpha"), Some('α'));
        assert_eq!(name_to_symbol("phi"), Some('φ'));
        assert_eq!(name_to_symbol("Phi"), Some('Φ'));
        assert_eq!(name_to_symbol("omega"), Some('ω'));
        assert_eq!(name_to_symbol("unknown"), None);
    }

    #[test]
    fn test_replace_greek_names() {
        assert_eq!(replace_greek_names("phi + alpha"), "φ + α");
        assert_eq!(replace_greek_names("sin(theta)"), "sin(θ)");
        assert_eq!(replace_greek_names("Gamma = delta"), "Γ = δ");
    }
}
