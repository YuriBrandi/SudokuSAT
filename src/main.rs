mod sudoku;

use std::{sync::mpsc::{self, Receiver}};
use eframe::{run_native, App, CreationContext, NativeOptions};

fn main() {

    let icon = include_bytes!("../assets/icon.png");
    let image = image::load_from_memory(icon).expect("Failed to open icon path").to_rgba8();
    let (icon_width, icon_height) = image.dimensions();

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_icon(egui::IconData {
                rgba: image.into_raw(), 
                width: icon_width, 
                height: icon_height,
            })
            .with_min_inner_size([400.0, 300.0]), // Minimum window size


        ..Default::default()
    };


    run_native(
        "Sudoku Solver",
        options,
        Box::new(|cc| Ok(Box::new(MatrixApp::new(cc)))),
    )
    .unwrap();
}

struct MatrixApp {
    matrix_size: usize,
    matrix: Vec<Vec<i8>>, // Matrix of 8-bit integers
    ui_scale: f32,
    dark_mode: bool, // Track light/dark mode
    invalid_poss: Vec<(usize, usize)>,
    show_correctness: bool,
    solution_time: f64,

    // Thread management
    rx_matrix: Option<Receiver<Vec<Vec<i8>>>>,
    rx_time: Option<Receiver<f64>>,
}

impl MatrixApp {
    fn new(_: &CreationContext<'_>) -> Self {
        Self {
            matrix_size: 3,
            matrix: vec![vec![0; 9]; 9],
            ui_scale: 1.,
            dark_mode: true,
            invalid_poss: Vec::new(),
            show_correctness: false,
            solution_time: f64::NAN,
            rx_matrix: None,
            rx_time: None
        }
    }

    fn update_matrix(&mut self) {
        self.matrix = vec![vec![0; self.matrix_size.pow(2)]; self.matrix_size.pow(2)];
        self.invalid_poss.clear();
        self.show_correctness = false;
        self.solution_time = f64::NAN;
    }
}

impl App for MatrixApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("right_panel")
        .max_width(350.)
        .show(ctx, |ui| {

            ctx.set_pixels_per_point(self.ui_scale);
            ctx.set_visuals( if self.dark_mode {egui::Visuals::dark()} else {egui::Visuals::light()});

            if ctx.input(|i| i.modifiers.ctrl || i.modifiers.mac_cmd) {
                if ctx.input(|i| i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals)) {
                    if self.ui_scale < 1. {self.ui_scale = 1.}
                    else if self.ui_scale < 2. {self.ui_scale += 0.5}
                } else if ctx.input(|i| i.key_pressed(egui::Key::Minus)) { // Ctrl -
                    if self.ui_scale == 1. {self.ui_scale = 0.8}
                    else if self.ui_scale > 1. {self.ui_scale -= 0.5}
                }
            }


            ui.label(
                egui::RichText::new("Settings")
                    .size(20.0)
                    .strong()
                    .monospace()
            );

            ui.add_space(15.);

            //Scrollable settings in case of overflow.
            egui::ScrollArea::vertical().show(ui, |ui|{

                ui.add(
                    egui::Checkbox::new(&mut self.dark_mode, "Dark mode")
                );

                ui.add_space(10.);

                if ui.add_enabled(self.rx_matrix.is_none(), egui::Slider::new(&mut self.matrix_size, 1..=5).text("Matrix Size")).changed() {
                    self.update_matrix();
                }

                ui.add_space(10.);

                //Show Ctrl/Cmd according to OS, using macos as target for cmd.
                egui::ComboBox::from_label(format!("Zoom factor {}", if cfg!(target_os = "macos") {"(Cmd -/+)"} else {"(Ctrl -/+)"}))
                .selected_text(format!("{:?}", self.ui_scale))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.ui_scale, 0.8, "Small");
                    ui.selectable_value(&mut self.ui_scale, 1., "Regular");
                    ui.selectable_value(&mut self.ui_scale, 1.5, "Big");
                    ui.selectable_value(&mut self.ui_scale, 2., "Huge");
                });

                ui.add_space(10.);

                ui.separator();

                ui.add_space(10.);

                //if(self.rx_matrix.is_none())

                ui.label(
                    egui::RichText::new("Operations")
                        .size(20.0)
                        .strong()
                        .monospace()
                );

                ui.add_space(10.);

                if ui.add_enabled(self.rx_matrix.is_none(), egui::Button::new("\u{1F3B2} Generate Random Puzzle")).clicked() {

                    // Creating a message channel for non-blocking matrix receive.
                    let (tx, rx) = mpsc::channel::<Vec<Vec<i8>>>();

                    // Reset matrix
                    self.update_matrix();

                    // Cloning self data since borrowing would escape from the method (error from compiler).
                    let mut matrix_clone = self.matrix.clone();
                    let seed_size = self.matrix_size.pow(2) * 2;

                    // Execute algorithm on a separate thread (still sequentially)
                    // This is needed to avoid GUI freezes for long computations.
                    std::thread::spawn(move || {
                        sudoku::generate_random_matrix(&mut matrix_clone, seed_size);
                        tx.send(matrix_clone).unwrap();
                    });

                    self.rx_matrix = Some(rx);

                }

                ui.add_space(10.);

                if ui.add_enabled(self.rx_matrix.is_none(), egui::Button::new("\u{1F504} Reset Grid")).clicked() {
                    self.update_matrix();
                }

                ui.add_space(10.);

                let sat_btn = ui.add_enabled(self.rx_matrix.is_none(), egui::Button::new("\u{2139} Show SAT Reduction"));


                egui::Popup::menu(&sat_btn)
                        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                        .show(|ui| {
                            ui.label(format!("SAT DIMACS CNF Form"));

                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .stick_to_bottom(true)
                                .show(ui, |ui| {
                                    ui.add(
                                        
                                    egui::Label::new(
                                            egui::RichText::new(sudoku::get_sat_decode(&mut self.matrix))
                                                //.size(14.0)
                                                .strong()
                                                .monospace()
                                        )
                                    );
                                });
                        });

                ui.add_space(10.);

                if ui.add_enabled(self.rx_matrix.is_none(), egui::Button::new("\u{2705} Check Solution")).clicked() {
                    let invalid_positions = sudoku::is_matrix_valid(&mut self.matrix);

                    self.invalid_poss = invalid_positions.clone();
                    self.show_correctness = true;

                    if invalid_positions.is_empty() {
                        println!("Correct solution");
                    }
                    else {
                        println!("Invalid values on: ");
                        for pos in invalid_positions {
                            println!(" ({}, {}), ", pos.0, pos.1);
                        }
                    }
                }

                ui.add_space(5.);

                if self.show_correctness {

                    ui.label(
                        egui::RichText::new(if self.invalid_poss.is_empty() {"\u{2705} Correct."} else {"\u{274C} invalid/blank cells."})
                            .size(14.0)
                            .strong()
                            .color(if self.invalid_poss.is_empty() {egui::Color32::DARK_GREEN} else {egui::Color32::DARK_RED})
                            .monospace()
                    );
                }
                
                ui.add_space(10.);


                ui.label(
                    egui::RichText::new("Right-click on a cell to edit its value")
                        .size(13.)
                        .italics()
                );

                ui.add_space(10.);

                ui.separator();

                ui.add_space(10.);

                ui.label(
                    egui::RichText::new("Solve")
                        .size(20.0)
                        .strong()
                        .monospace()
                );

                ui.add_space(10.);


                if ui.add_enabled(self.rx_matrix.is_none(), egui::Button::new("\u{26A1} Solve Backtrack")).clicked()  {

                    // Creating a message channel for non-blocking matrix receive.
                    let (tx_matrix, rx_matrix) = mpsc::channel::<Vec<Vec<i8>>>();

                    // Creating another message channel for non-blocking time receive.
                    let (tx_time, rx_time) = mpsc::channel::<f64>();

                    // Cloning self data since borrowing would escape from the method (error from compiler).
                    let mut matrix_clone = self.matrix.clone();
 
                    // Execute algorithm on a separate thread (still sequentially)
                    // This is needed to avoid GUI freezes for long computations.
                    std::thread::spawn(move || {
                        tx_time.send(sudoku::solve_backtracking_time(&mut matrix_clone)).unwrap();
                        tx_matrix.send(matrix_clone).unwrap();
                    });
 
                    self.rx_matrix = Some(rx_matrix);
                    self.rx_time = Some(rx_time);

                }

                ui.add_space(10.);

                if ui.add_enabled(self.rx_matrix.is_none(), egui::Button::new("\u{26A1} Solve SAT")).clicked()  {

                    // Creating a message channel for non-blocking matrix receive.
                   let (tx_matrix, rx_matrix) = mpsc::channel::<Vec<Vec<i8>>>();

                    // Creating another message channel for non-blocking time receive.
                    let (tx_time, rx_time): (mpsc::Sender<f64>, mpsc::Receiver<f64>) = mpsc::channel();

                    // Cloning self data since borrowing would escape from the method (error from compiler).
                    let mut matrix_clone = self.matrix.clone();

                    // Execute algorithm on a separate thread (still sequentially)
                    // This is needed to avoid GUI freezes for long computations.
                    std::thread::spawn(move || {
                        tx_time.send(sudoku::solve_sat_time(&mut matrix_clone)).unwrap();
                        tx_matrix.send(matrix_clone).unwrap();
                    });

                    self.rx_matrix = Some(rx_matrix);
                    self.rx_time = Some(rx_time);

               }

                ui.add_space(5.);

                 if !self.solution_time.is_nan() {

                    ui.label(
                        egui::RichText::new(if self.solution_time.is_finite() {format!("Solution found in {:.3} s.", self.solution_time)} else {"\u{274C} Puzzle is unsolvable.".to_string()})
                            .size(14.0)
                            .strong()
                            .color(if self.solution_time.is_finite() {egui::Color32::DARK_GREEN} else {egui::Color32::DARK_RED})
                            .monospace()
                    );
                }

                ui.separator();

                ui.add_space(10.);
                                
                if self.rx_matrix.is_some(){
                    ui.spinner();
                }





                // Check completition (if there is any) with non-blocking receive
                if let Some(rx) = &self.rx_matrix {
                    if let Ok(new_matrix) = rx.try_recv() {
                        self.matrix = new_matrix;
                        println!("Received computation.");
                        self.rx_matrix = None;
                    }
                }

                // Check completition (if there is any) with non-blocking receive
                if let Some(rx) = &self.rx_time {
                    if let Ok(elap_time) = rx.try_recv() {
                        self.solution_time = elap_time;
                        println!("Received time.");
                        self.rx_time = None;
                    }
                }

            });

        });

        egui::CentralPanel::default().show(ctx, |ui| {

            ui.label(
                egui::RichText::new("Sudoku Grid")
                    .size(20.0)
                    .strong()
                    .monospace()
            );
            
            ui.add_space(25.);


            /*
                Vertical alignment not working as expected for Grids.
                Using ScrollArea in case of overflowing content.
                ScrollArea does not show otherwise by default.
             */
            egui::ScrollArea::both().show(ui,|ui| {

                // Draw the matrix with a grid and borders
                egui::Grid::new("matrix_grid")
                    //.striped(true)
                    .spacing([4., 4.])
                    .show(ui, |ui| {
                        // Cycle by index and not by value to avoid borrowing issues
                        for row_index in 0..self.matrix_size.pow(2) {
                            for col_index in 0..self.matrix_size.pow(2) {
                        //for (row_index, row) in &mut self.matrix.iter().enumerate() {
                          //  for (col_index, value) in row.iter().enumerate() {
                                
                                ui.push_id((row_index, col_index), |ui| {

                                    let resp = ui.interact(ui.max_rect(), ui.id(), egui::Sense::click());

                                    // Draw each cell with a border
                                    ui.vertical_centered(|ui| {
                                        egui::Frame::new()
                                        // Integer quotient represents block group. % 2 alternates each group.
                                        .fill(if (row_index / self.matrix_size) % 2 == (col_index / self.matrix_size) % 2  {ui.visuals().warn_fg_color} else {ui.visuals().widgets.inactive.bg_fill})
                                        .stroke(egui::Stroke::new(
                                            2.0,
                                            if resp.hovered()
                                                {ui.visuals().widgets.active.bg_stroke.color} else {egui::Color32::TRANSPARENT}))
                                        .inner_margin(egui::Margin {
                                            left: 8,
                                            right: 8,
                                            top: 10,
                                            bottom: 10})
                                        .show(ui, |ui|{
                                            let value = self.matrix[row_index][col_index];
                                            ui.add(egui::Label::new(
                                                egui::RichText::new(if value > 0 {format!("{}", value)} else {String::from(" ")}) 
                                                .color(if self.invalid_poss.contains(&(row_index, col_index)) {ui.visuals().error_fg_color} else {ui.visuals().strong_text_color()})
                                                .size(16.0)
                                                .strong()
                                            ).selectable(false))
                                        });

                                        let popup_id = ui.make_persistent_id("edit_popup");
                                        
                                        if resp.secondary_clicked() {
                                            //ui.memory_mut(|mem| mem.open_popup(popup_id));
                                            egui::Popup::open_id(ctx, popup_id);       
                                        }

                                        egui::Popup::menu(&resp)
                                            .id(popup_id)
                                            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                                            .show(|ui| {
                                                //println!("Popup ID: {:?}", popup_id);
                                                ui.label(format!("Changing value of ({}, {})", row_index, col_index));


                                                ui.add(egui::Slider::new(&mut self.matrix[row_index][col_index], 0..=self.matrix_size.pow(2) as i8));

                                                // Disable solution check colors
                                                self.show_correctness = false;
                                                self.invalid_poss.clear();
                                            });


                                    });
                                });
                            }
                            ui.end_row();

                        }
                    });

            });
        });
    }
}