//! XYZ / CSV point-list parsing — the simplest, least-standardized
//! export: one point per line, `x y z` or `x,y,z`, optionally followed
//! by `r g b`. No header row is assumed (the format has none by
//! convention); a line that fails to parse as all-numeric is treated as
//! a header/comment and skipped rather than erroring, since real-world
//! XYZ exports vary on this.

use crate::{ParseError, PointCloud};

pub fn parse_xyz(text: &str) -> Result<PointCloud, ParseError> {
    let mut points = Vec::new();
    let mut color_rows: Vec<[u8; 3]> = Vec::new();
    let mut any_colors = false;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|f| !f.is_empty())
            .collect();
        if fields.len() < 3 {
            continue;
        }
        let parsed: Result<Vec<f64>, _> = fields.iter().take(3).map(|f| f.parse::<f64>()).collect();
        let Ok(xyz) = parsed else {
            // Not all-numeric -- treat as a header/comment row, not an error.
            continue;
        };
        points.push([xyz[0], xyz[1], xyz[2]]);

        if fields.len() >= 6 {
            let rgb: Result<Vec<u8>, _> = fields[3..6].iter().map(|f| f.parse::<u8>()).collect();
            if let Ok(rgb) = rgb {
                color_rows.push([rgb[0], rgb[1], rgb[2]]);
                any_colors = true;
                continue;
            }
        }
        color_rows.push([0, 0, 0]);
    }

    Ok(PointCloud {
        points,
        colors: if any_colors { Some(color_rows) } else { None },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_whitespace_separated_xyz() {
        let cloud = parse_xyz("0 0 0\n1 2 3\n4 5 6\n").unwrap();
        assert_eq!(
            cloud.points,
            vec![[0.0, 0.0, 0.0], [1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]
        );
        assert!(cloud.colors.is_none());
    }

    #[test]
    fn parses_comma_separated_xyz_with_colors() {
        let cloud = parse_xyz("0,0,0,255,0,0\n1,1,1,0,255,0\n").unwrap();
        assert_eq!(cloud.points.len(), 2);
        assert_eq!(cloud.colors.unwrap(), vec![[255, 0, 0], [0, 255, 0]]);
    }

    #[test]
    fn skips_a_non_numeric_header_row() {
        let cloud = parse_xyz("x,y,z\n1,2,3\n").unwrap();
        assert_eq!(cloud.points, vec![[1.0, 2.0, 3.0]]);
    }

    #[test]
    fn ignores_blank_and_comment_lines() {
        let cloud = parse_xyz("# comment\n\n1 2 3\n").unwrap();
        assert_eq!(cloud.points, vec![[1.0, 2.0, 3.0]]);
    }

    #[test]
    fn empty_input_is_an_empty_cloud_not_an_error() {
        let cloud = parse_xyz("").unwrap();
        assert!(cloud.points.is_empty());
    }
}
