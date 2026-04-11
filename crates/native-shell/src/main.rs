#[derive(Clone, Copy)]
enum Screen {
    Dashboard,
    Planning,
    Atomization,
    Monitoring,
}

#[derive(Default)]
struct DashboardService;
#[derive(Default)]
struct PlanningService;
#[derive(Default)]
struct AtomizationService;
#[derive(Default)]
struct MonitoringService;

impl DashboardService {
    fn render(&self) -> String {
        "dashboard".to_string()
    }
}

impl PlanningService {
    fn render(&self) -> String {
        "planning".to_string()
    }
}

impl AtomizationService {
    fn render(&self) -> String {
        "atomization".to_string()
    }
}

impl MonitoringService {
    fn render(&self) -> String {
        "monitoring".to_string()
    }
}

#[derive(Default)]
struct NativeShellServices {
    dashboard: DashboardService,
    planning: PlanningService,
    atomization: AtomizationService,
    monitoring: MonitoringService,
}

impl NativeShellServices {
    fn render(&self, screen: Screen) -> String {
        match screen {
            Screen::Dashboard => self.dashboard.render(),
            Screen::Planning => self.planning.render(),
            Screen::Atomization => self.atomization.render(),
            Screen::Monitoring => self.monitoring.render(),
        }
    }
}

#[derive(Default)]
struct NativeShellApp {
    services: NativeShellServices,
}

impl NativeShellApp {
    fn launch(&self) -> Vec<String> {
        vec![
            self.services.render(Screen::Dashboard),
            self.services.render(Screen::Planning),
            self.services.render(Screen::Atomization),
            self.services.render(Screen::Monitoring),
        ]
    }
}

fn main() {
    let native_shell = NativeShellApp::default();
    let startup_screens = native_shell.launch();
    drop(startup_screens);
}
