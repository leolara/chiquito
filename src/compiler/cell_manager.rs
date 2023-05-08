use std::{collections::HashMap, fmt::Debug, rc::Rc};

use crate::ast::{ForwardSignal, InternalSignal, StepType};

use super::{Column, CompilationUnit};

#[derive(Clone, Debug)]
pub struct SignalPlacement {
    pub column: Column,
    pub rotation: i32,
}

#[derive(Debug, Clone)]
pub struct StepPlacement {
    height: u32,
    signals: HashMap<InternalSignal, SignalPlacement>,
}

#[derive(Debug, Clone)]
pub struct Placement<F, StepArgs> {
    pub forward: HashMap<ForwardSignal, SignalPlacement>,
    pub steps: HashMap<Rc<StepType<F, StepArgs>>, StepPlacement>,
    pub columns: Vec<Column>,
}

impl<F, StepArgs> Default for Placement<F, StepArgs> {
    fn default() -> Self {
        Self {
            forward: Default::default(),
            steps: Default::default(),
            columns: Default::default(),
        }
    }
}

impl<F, StepArgs> Placement<F, StepArgs> {
    pub fn get_forward_placement(&self, forward: &ForwardSignal) -> SignalPlacement {
        self.forward
            .get(forward)
            .expect("forward signal not found")
            .clone()
    }

    pub fn find_internal_signal_placement(
        &self,
        step: &StepType<F, StepArgs>,
        signal: &InternalSignal,
    ) -> SignalPlacement {
        self.steps
            .get(step)
            .expect("step not found")
            .signals
            .get(signal)
            .expect("signal not found")
            .clone()
    }

    pub fn step_height(&self, step: &StepType<F, StepArgs>) -> u32 {
        self.steps.get(step).expect("step not found").height
    }
}

pub trait CellManager {
    fn place<F, StepArgs>(&self, unit: &mut CompilationUnit<F, StepArgs>);
}

#[derive(Debug, Default)]
pub struct SingleRowCellManager {}

impl CellManager for SingleRowCellManager {
    fn place<F, StepArgs>(&self, unit: &mut CompilationUnit<F, StepArgs>) {
        let mut placement = Placement::<F, StepArgs> {
            forward: HashMap::new(),
            steps: HashMap::new(),
            columns: Vec::new(),
        };

        let mut forward_signals: u32 = 0;

        for forward_signal in unit.forward_signals.iter() {
            let column = if let Some(annotation) = unit.annotations.get(&forward_signal.uuid()) {
                Column::advice(
                    format!("srcm forward {}", annotation),
                    forward_signal.phase(),
                )
            } else {
                Column::advice("srcm forward", forward_signal.phase())
            };

            placement.columns.push(column.clone());

            placement.forward.insert(
                *forward_signal,
                SignalPlacement {
                    column,
                    rotation: 0,
                },
            );

            forward_signals += 1;
        }

        let mut max_internal_width: u32 = 0;

        for step in unit.step_types.values() {
            let mut internal_signals: u32 = 0;

            let mut step_placement = StepPlacement {
                height: 1, // This cellmanager always have SuperRows with height 1
                signals: HashMap::new(),
            };

            for signal in step.signals.iter() {
                let column_pos = forward_signals + internal_signals;
                let column = if placement.columns.len() <= column_pos as usize {
                    let column = if let Some(annotation) = unit.annotations.get(&signal.uuid()) {
                        Column::advice(format!("srcm internal signal {}", annotation), 0)
                    } else {
                        Column::advice("srcm internal signal", 0)
                    };

                    placement.columns.push(column.clone());
                    column
                } else {
                    placement.columns[column_pos as usize].clone()
                };

                step_placement.signals.insert(
                    *signal,
                    SignalPlacement {
                        column,
                        rotation: 0,
                    },
                );

                internal_signals += 1;
            }

            placement.steps.insert(Rc::clone(step), step_placement);
            max_internal_width = max_internal_width.max(internal_signals);
        }

        unit.columns.extend_from_slice(&placement.columns);
        unit.placement = placement;
    }
}

#[derive(Debug, Default)]
pub struct MaxWidthCellManager {
    max_width: usize,
}

impl CellManager for MaxWidthCellManager {
    fn place<F, StepArgs>(&self, unit: &mut CompilationUnit<F, StepArgs>) {
        let mut placement = Placement::<F, StepArgs> {
            forward: HashMap::new(),
            steps: HashMap::new(),
            columns: Vec::new(),
        };

        let mut forward_signal_column: usize = 0;
        let mut forward_signal_row: usize = 0;

        for forward_signal in unit.forward_signals.iter() {
            let column = if placement.columns.len() <= forward_signal_column {
                let column = if let Some(annotation) = unit.annotations.get(&forward_signal.uuid())
                {
                    Column::advice(format!("mwcm forward signal {}", annotation), 0)
                } else {
                    Column::advice("mwcm forward signal", 0)
                };

                placement.columns.push(column.clone());
                column
            } else {
                placement.columns[forward_signal_column].clone()
            };

            placement.forward.insert(
                *forward_signal,
                SignalPlacement {
                    column,
                    rotation: forward_signal_row as i32,
                },
            );

            forward_signal_column += 1;
            if forward_signal_column >= self.max_width {
                forward_signal_column = 0;
                forward_signal_row += 1;
            }
        }

        for step in unit.step_types.values() {
            let mut step_placement = StepPlacement {
                height: if forward_signal_column > 0 {
                    (forward_signal_row + 1) as u32
                } else {
                    forward_signal_row as u32
                },
                signals: HashMap::new(),
            };

            let mut internal_signal_column = forward_signal_column;
            let mut internal_signal_row = forward_signal_row;

            for signal in step.signals.iter() {
                let column = if placement.columns.len() <= internal_signal_column {
                    let column = if let Some(annotation) = unit.annotations.get(&signal.uuid()) {
                        Column::advice(format!("mwcm forward signal {}", annotation), 0)
                    } else {
                        Column::advice("mwcm forward signal", 0)
                    };

                    placement.columns.push(column.clone());
                    column
                } else {
                    placement.columns[internal_signal_column].clone()
                };

                step_placement.signals.insert(
                    *signal,
                    SignalPlacement {
                        column,
                        rotation: internal_signal_row as i32,
                    },
                );

                step_placement.height = (internal_signal_row + 1) as u32;

                internal_signal_column += 1;
                if internal_signal_column >= self.max_width {
                    internal_signal_column = 0;
                    internal_signal_row += 1;
                }
            }

            placement.steps.insert(Rc::clone(step), step_placement);
        }

        unit.columns.extend_from_slice(&placement.columns);
        unit.placement = placement;
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::{
        ast::{ForwardSignal, StepType},
        compiler::CompilationUnit,
    };

    use super::{CellManager, MaxWidthCellManager};

    #[test]
    fn test_max_width_cm_2_columns() {
        let mut unit = CompilationUnit::<(), ()> {
            forward_signals: vec![
                ForwardSignal::new_with_phase(0, "a".to_string()),
                ForwardSignal::new_with_phase(0, "a".to_string()),
            ],
            ..Default::default()
        };

        let mut step1 = StepType::new(1500, "step1".to_string());
        step1.add_signal("c1");
        step1.add_signal("d");
        step1.add_signal("e");
        let mut step2 = StepType::new(1501, "step2".to_string());
        step2.add_signal("c2");

        let step1 = Rc::new(step1);
        let step2 = Rc::new(step2);

        unit.step_types.insert(1500, Rc::clone(&step1));
        unit.step_types.insert(1501, Rc::clone(&step2));

        // forward signals: a, b; step1 internal: c1, d, e; step2 internal c2

        let cm = MaxWidthCellManager { max_width: 2 };

        cm.place(&mut unit);

        assert_eq!(unit.placement.columns.len(), 2);
        assert_eq!(unit.placement.forward.len(), 2);
        assert_eq!(
            unit.placement
                .steps
                .get(&step1)
                .expect("should be there")
                .height,
            3
        );
        assert_eq!(
            unit.placement
                .steps
                .get(&step1)
                .expect("should be there")
                .signals
                .len(),
            3
        );
        assert_eq!(
            unit.placement
                .steps
                .get(&step2)
                .expect("should be there")
                .height,
            2
        );
        assert_eq!(
            unit.placement
                .steps
                .get(&step2)
                .expect("should be there")
                .signals
                .len(),
            1
        );
    }
}
