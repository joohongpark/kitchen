//! 단위 변환. 무게·부피는 같은 계열끼리 변환하고, 개수 단위는 같은 단위끼리만 비교한다.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dimension {
    Mass,
    Volume,
    /// 개, 모, 단 등. 문자열이 같아야 비교 가능
    Count,
}

/// (기본 단위로 환산한 배율, 계열). 무게는 g, 부피는 ml 기준.
fn lookup(unit: &str) -> Option<(f64, Dimension)> {
    let u = unit.trim().to_lowercase();
    let r = match u.as_str() {
        "g" | "그램" => (1.0, Dimension::Mass),
        "kg" | "킬로" | "킬로그램" => (1000.0, Dimension::Mass),
        "ml" | "밀리리터" | "cc" => (1.0, Dimension::Volume),
        "l" | "리터" => (1000.0, Dimension::Volume),
        "큰술" | "큰 술" | "tbsp" => (15.0, Dimension::Volume),
        "작은술" | "작은 술" | "tsp" => (5.0, Dimension::Volume),
        "컵" | "cup" => (200.0, Dimension::Volume),
        _ => return None,
    };
    Some(r)
}

pub fn normalize(unit: &str) -> String {
    unit.trim().to_string()
}

/// `qty`를 `from` 단위에서 `to` 단위로 바꾼다. 변환할 수 없으면 None.
pub fn convert(qty: f64, from: &str, to: &str) -> Option<f64> {
    let (from_n, to_n) = (normalize(from), normalize(to));
    if from_n.eq_ignore_ascii_case(&to_n) {
        return Some(qty);
    }
    let (fa, fd) = lookup(&from_n)?;
    let (ta, td) = lookup(&to_n)?;
    (fd == td).then(|| qty * fa / ta)
}

pub fn dimension(unit: &str) -> Dimension {
    lookup(unit).map(|(_, d)| d).unwrap_or(Dimension::Count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_within_dimension() {
        assert_eq!(convert(2.0, "큰술", "ml"), Some(30.0));
        assert_eq!(convert(1.5, "kg", "g"), Some(1500.0));
        assert_eq!(convert(1.0, "모", "모"), Some(1.0));
    }

    #[test]
    fn rejects_cross_dimension() {
        assert_eq!(convert(1.0, "g", "ml"), None);
        assert_eq!(convert(1.0, "모", "g"), None);
        assert_eq!(convert(1.0, "개", "단"), None);
    }
}
