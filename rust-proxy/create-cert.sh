#!/bin/sh

# Remove existing keys directory and create a new one
rm -rf keys
mkdir -p keys
cd keys/

# Step 1: Create an unencrypted private key and a CSR (Certificate Signing Request)
openssl req -newkey rsa:2048 -nodes -subj "/C=FI/CN=localhost" -keyout key.pem -out key.csr

# Step 2: Create a self-signed root CA
openssl req -x509 -sha256 -nodes -subj "/C=FI/CN=MyRootCA" -days 1825 -newkey rsa:2048 -keyout rootCA.key -out rootCA.crt

# Step 3: Create a configuration file for SAN (Subject Alternative Name)
cat <<EOF > localhost.ext
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names

[alt_names]
DNS.1 = server
IP.1 = 127.0.0.1
EOF

# Step 4: Sign the CSR with the Root CA and create the final certificate (cert.pem)
openssl x509 -req -in key.csr -CA rootCA.crt -CAkey rootCA.key -CAcreateserial -out cert.pem -days 365 -extfile localhost.ext