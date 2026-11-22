#!/bin/bash
# Run as root in the VM, once: install svrngs (both accounts' login service),
# the keeper, the relay and Alex's boot request; Sam's model. Sam's own steps
# are the run's, in Sam's terminal (run.py).
set -e; cd /root/stage
install -m755 svrngs svrngs-keeper /usr/local/bin/
install -m644 svrngs.service /etc/systemd/user/svrngs.service
install -m644 svrngs-relay.service /etc/systemd/system/
install -d -o sam -g sam /home/sam/models && install -o sam -g sam -m644 model.gguf /home/sam/models/qwen2.5-0.5b.gguf
install -d -o alex -g alex /home/alex/asks /home/alex/.config/systemd/user
install -o alex -g alex ask.py /home/alex/ask.py
install -o alex -g alex -m644 alex-asks.service /home/alex/.config/systemd/user/
systemctl daemon-reload && systemctl enable --now svrngs-relay
loginctl enable-linger alex; loginctl show-user sam -p Linger; loginctl show-user alex -p Linger
A="systemd-run -M alex@.host --user --wait --pipe -q"
$A systemctl --user daemon-reload
$A systemctl --user enable --now svrngs.service
$A systemctl --user enable alex-asks.service
command -v llama-server; rpm -q llama-cpp
