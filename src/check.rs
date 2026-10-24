use ratatui::crossterm::terminal;

use crate::{
    graph::{CellWidthType, Graph},
    GraphWidthType, Result,
};

pub fn decide_cell_width_type(
    graph: &Graph,
    cell_width_type: Option<GraphWidthType>,
    allow_horizontal_scroll: bool,
) -> Result<CellWidthType> {
    let (w, h) = terminal::size()?;
    if allow_horizontal_scroll {
        return Ok(viewport_cell_width_type(
            graph.max_pos_x,
            w,
            cell_width_type,
        ));
    }
    decide_cell_width_type_from(graph.max_pos_x, w as usize, h as usize, cell_width_type)
}

pub fn graph_viewport_width(terminal_width: u16) -> u16 {
    (terminal_width / 3).max(1)
}

fn viewport_cell_width_type(
    max_pos_x: usize,
    terminal_width: u16,
    requested: Option<GraphWidthType>,
) -> CellWidthType {
    match requested {
        Some(GraphWidthType::Double) => CellWidthType::Double,
        Some(GraphWidthType::Single) => CellWidthType::Single,
        _ if (max_pos_x + 1) * 2 <= usize::from(graph_viewport_width(terminal_width)) => {
            CellWidthType::Double
        }
        _ => CellWidthType::Single,
    }
}

fn decide_cell_width_type_from(
    max_pos_x: usize,
    term_width: usize,
    term_height: usize,
    cell_width_type: Option<GraphWidthType>,
) -> Result<CellWidthType> {
    let single_image_cell_width = max_pos_x + 1;
    let double_image_cell_width = single_image_cell_width * 2;

    match cell_width_type {
        Some(GraphWidthType::Double) => {
            let required_width = double_image_cell_width + 2;
            if required_width > term_width {
                let msg = format!("Terminal too small ({term_width}x{term_height} characters). The current graph needs at least {required_width} columns to display properly.");
                return Err(msg.into());
            }
            Ok(CellWidthType::Double)
        }
        Some(GraphWidthType::Single) => {
            let required_width = single_image_cell_width + 2;
            if required_width > term_width {
                let msg = format!("Terminal too small ({term_width}x{term_height} characters). The current graph needs at least {required_width} columns to display properly.");
                return Err(msg.into());
            }
            Ok(CellWidthType::Single)
        }
        Some(GraphWidthType::Auto) | None => {
            let double_required_width = double_image_cell_width + 2;
            if double_required_width <= term_width {
                return Ok(CellWidthType::Double);
            }
            let single_required_width = single_image_cell_width + 2;
            if single_required_width <= term_width {
                return Ok(CellWidthType::Single);
            }
            let msg = format!("Terminal too small ({term_width}x{term_height} characters). The current graph needs at least {single_required_width} columns to display properly.");
            Err(msg.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_sixel_graph_can_scroll_in_a_normal_terminal() {
        assert_eq!(
            viewport_cell_width_type(253, 193, None),
            CellWidthType::Single
        );
        assert_eq!(graph_viewport_width(193), 64);
        assert!(decide_cell_width_type_from(253, 193, 52, None).is_err());
    }

    #[test]
    fn small_graph_and_explicit_width_keep_their_cell_sizes() {
        assert_eq!(
            viewport_cell_width_type(3, 120, None),
            CellWidthType::Double
        );
        assert_eq!(
            viewport_cell_width_type(253, 193, Some(GraphWidthType::Double)),
            CellWidthType::Double
        );
        assert_eq!(graph_viewport_width(0), 1);
    }
}
