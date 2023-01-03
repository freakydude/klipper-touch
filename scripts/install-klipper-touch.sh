#!/bin/bash
LOG_PATH="/tmp/klipper-touch.log"
SYSTEMDDIR="/etc/systemd/system"

install_script()
{
# Install dependencies
#    curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
#    sudo apt-get install -y nodejs
    npm i --prefix ${SRCDIR}   #to force use ci instead of i, if configured write, enable_node_updates: true in moonraker.conf does the job

# Create systemd service file
    SERVICE_FILE="${SYSTEMDDIR}/klipper-touch.service"
    [ -f $SERVICE_FILE ] && [ "$FORCE_DEFAULTS" = "n" ] && return
    report_status "Installing system start script..."
    sudo /bin/sh -c "cat > ${SERVICE_FILE}" << EOF
#Systemd service file for klipper-touch
[Unit]
Description=Webbased touch UI for Klipper
Requires=network-online.target
After=network-online.target

[Install]
WantedBy=multi-user.target

[Service]
Type=simple
User=$USER
RemainAfterExit=yes
WorkingDirectory=${SRCDIR}
#ExecStart=${LAUNCH_CMD} -l ${LOG_PATH}
ExecStart=${LAUNCH_CMD} 2>&1| tee ${LOG_PATH}
Restart=always
RestartSec=10
EOF
# Use systemctl to enable the klipper systemd service script
    sudo systemctl enable klipper-touch.service
    sudo systemctl daemon-reload
}


start_software()
{
    report_status "Launching klipper-touch..."
    sudo systemctl restart klipper-touch.service
}

# Helper functions
report_status()
{
    echo -e "\n\n###### $1"
}

verify_ready()
{
    if [ "$EUID" -eq 0 ]; then
        echo "This script must not run as root"
        exit -1
    fi
}

# Force script to exit if an error occurs
set -e

# Find SRCDIR from the pathname of this script
SRCDIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )"/.. && pwd )"
LAUNCH_CMD="/usr/bin/npm run dev --prefix ${SRCDIR}"


# Parse command line arguments
while getopts "rfc:l:" arg; do
    case $arg in
        r) REBUILD_ENV="y";;
        f) FORCE_DEFAULTS="y";;
        l) LOG_PATH=$OPTARG;;
    esac
done

# Run installation steps defined above
verify_ready
install_script
start_software
