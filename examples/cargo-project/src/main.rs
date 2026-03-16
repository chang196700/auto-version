// Version information injected by auto-version at build time.
// Access via the env!() macro.

fn main() {
    println!("Version:      {}", env!("VERSION"));
    println!("Full SemVer:  {}", env!("VERSION_FULL"));
    println!("Info:         {}", env!("VERSION_INFO"));
    println!("Git SHA:      {}", env!("GIT_SHORT_SHA"));
    println!("Branch:       {}", env!("GIT_BRANCH"));
    println!("Build date:   {}", env!("BUILD_DATE"));
}
