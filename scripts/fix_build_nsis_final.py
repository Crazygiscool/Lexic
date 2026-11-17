content = open('crates/lexic-gui/src/views/grammar.rs').read() if False else ''
# Read current build script
content = open('crates/lexic-gui/scripts/build-nsis.sh').read()
print("Current build-nsis.sh:")
print(content[:2000])