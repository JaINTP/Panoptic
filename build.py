#!/usr/bin/env python3
"""Build script for Panoptic.

Produces Linux and Windows builds of the Tauri application.
"""

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path
from typing import NoReturn, Optional

# Constants
UI_DIR: Path = Path(__file__).parent / "crates" / "ui" / "panoptic-gui"
SRC_TAURI_DIR: Path = UI_DIR / "src-tauri"


def print_step(message: str) -> None:
    """Print a styled build step message."""
    print(f"\033[1;34m==>\033[0m \033[1;37m{message}\033[0m")


def print_success(message: str) -> None:
    """Print a success message."""
    print(f"\033[1;32m✓\033[0m \033[1;37m{message}\033[0m")


def print_error(message: str) -> None:
    """Print an error message."""
    print(f"\033[1;31mError:\033[0m {message}", file=sys.stderr)


def fail(message: str) -> NoReturn:
    """Print error and exit."""
    print_error(message)
    sys.exit(1)


def run_command(
    args: list[str], cwd: Optional[Path] = None, env: Optional[dict[str, str]] = None
) -> None:
    """Execute a shell command, showing output in real-time."""
    cmd_str = " ".join(args)
    print(f"\033[90m$ {cmd_str}\033[0m")
    try:
        subprocess.run(args, cwd=cwd, env=env, check=True)
    except subprocess.CalledProcessError as e:
        fail(f"Command failed with exit code {e.returncode}: {cmd_str}")


def check_tool(name: str) -> bool:
    """Check if a tool exists in PATH."""
    return shutil.which(name) is not None


def ensure_host_dependencies() -> None:
    """Ensure host tools like npm, cargo are available."""
    if not check_tool("npm"):
        fail("NPM is not installed. Please install Node.js/NPM.")
    if not check_tool("cargo"):
        fail("Cargo/Rust is not installed. Please install Rust.")


def build_frontend() -> None:
    """Install dependencies and build the frontend assets."""
    print_step("Installing frontend dependencies...")
    run_command(["npm", "install"], cwd=UI_DIR)

    print_step("Building frontend assets...")
    run_command(["npm", "run", "build"], cwd=UI_DIR)


def build_linux(bundle: bool) -> None:
    """Build the application natively for Linux."""
    print_step("Building natively for Linux...")
    cmd = ["npx", "tauri", "build"]
    if not bundle:
        cmd.append("--no-bundle")
    run_command(cmd, cwd=UI_DIR)
    print_success("Linux build completed successfully.")


def build_windows_local(bundle: bool) -> None:
    """Build the application for Windows locally using cargo-xwin."""
    print_step("Checking target x86_64-pc-windows-msvc...")
    # Add target if not present
    run_command(["rustup", "target", "add", "x86_64-pc-windows-msvc"])

    # Check for cargo-xwin
    print_step("Checking for cargo-xwin...")
    has_xwin = False
    try:
        res = subprocess.run(
            ["cargo", "xwin", "--version"],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        if res.returncode == 0:
            has_xwin = True
    except FileNotFoundError:
        pass

    if not has_xwin:
        print_step("Installing cargo-xwin...")
        run_command(["cargo", "install", "cargo-xwin", "--locked"])

    print_step("Building for Windows (x86_64-pc-windows-msvc) locally...")
    cmd = [
        "npx",
        "tauri",
        "build",
        "--runner",
        "cargo-xwin",
        "--target",
        "x86_64-pc-windows-msvc",
    ]
    if not bundle:
        cmd.append("--no-bundle")
    run_command(cmd, cwd=UI_DIR)
    print_success("Windows build completed successfully.")


def build_windows_docker(bundle: bool) -> None:
    """Build the application for Windows using Docker/Podman container."""
    print_step("Building for Windows using Docker...")

    if not (check_tool("docker") or check_tool("podman")):
        fail("Neither Docker nor Podman is installed.")

    engine = "docker" if check_tool("docker") else "podman"
    print_step(f"Using container engine: {engine}")

    docker_image = "liudonghua123/tauri-build:latest"

    # We map the cargo cache and build workspace to speed up subsequent builds
    cargo_home = Path.home() / ".cargo"
    cargo_home.mkdir(parents=True, exist_ok=True)

    # Prepare current dir absolute path
    workspace = Path(__file__).parent.resolve()

    uid = os.getuid()
    gid = os.getgid()

    apt_packages = "llvm clang lld nsis" if bundle else "llvm clang lld"
    bundle_flag = "" if bundle else "--no-bundle"

    cmd = [
        engine,
        "run",
        "--rm",
        "-v",
        f"{workspace}:/workspace",
        "-v",
        f"{cargo_home}:/root/.cargo",
        "-w",
        "/workspace/crates/ui/panoptic-gui",
        docker_image,
        "bash",
        "-c",
        f"export PATH=\"/root/.nvm/versions/node/v20.18.0/bin:/root/.cargo/bin:$PATH\" && apt-get update && apt-get install -y {apt_packages} && rustup update stable && rustup default stable && (which cargo-xwin || cargo install cargo-xwin) && npm install && npm run build && rustup target add x86_64-pc-windows-msvc && npx tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc {bundle_flag} && chown -R {uid}:{gid} /workspace",
    ]

    try:
        run_command(cmd)
        print_success("Windows build via Docker completed successfully.")
    finally:
        print_step("Restoring host file ownership permissions...")
        cleanup_cmd = [
            engine,
            "run",
            "--rm",
            "-v",
            f"{workspace}:/workspace",
            docker_image,
            "chown",
            "-R",
            f"{uid}:{gid}",
            "/workspace",
        ]
        try:
            subprocess.run(
                cleanup_cmd,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                check=True,
            )
        except subprocess.CalledProcessError:
            print_error("Failed to restore some host file permissions.")


def main() -> None:
    """Main execution entrypoint."""
    parser = argparse.ArgumentParser(description="Panoptic App Multi-Platform Builder")
    parser.add_argument(
        "--linux", action="store_true", help="Produce Linux build (AppImage/Deb)"
    )
    parser.add_argument(
        "--windows", action="store_true", help="Produce Windows build"
    )
    parser.add_argument(
        "--win-method",
        choices=["local", "docker"],
        default="local",
        help="Compilation method for Windows (default: local/cargo-xwin)",
    )
    parser.add_argument(
        "--bundle",
        action="store_true",
        help="Produce packaged installers (requires native compilers like NSIS locally or inside Docker)",
    )
    parser.add_argument(
        "--all", action="store_true", help="Build for all platforms (default if none specified)"
    )

    args = parser.parse_args()

    # If no flags are set, build all
    if not (args.linux or args.windows):
        args.all = True

    ensure_host_dependencies()
    build_frontend()

    if args.linux or args.all:
        build_linux(args.bundle)

    if args.windows or args.all:
        if args.win_method == "local":
            build_windows_local(args.bundle)
        else:
            build_windows_docker(args.bundle)


if __name__ == "__main__":
    main()
