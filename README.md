# Upload File

### Deployment Guide

##### Building the canisters
```bash
# makes the bash file executable
chmod +x scripts/build_script.sh

# builds the canister and generates the candid file
./scripts/build_script.sh

npm install

dfx deploy storage --argument '(false)'

npm run test
```