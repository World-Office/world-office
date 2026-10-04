# word-office-opencloud Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Transform word-office-opencloud from a bare prototype into a production-ready deployment companion that orchestrates OCIS + Word-Office Document Server via WOPI.

**Architecture:** Docker Compose-based deployment with configuration generation, health checks, and Word-Office branding. The app generates OCIS + Document Server configs from a single .env file and provides a setup wizard + status dashboard.

**Tech Stack:** Node.js 20+, Express 4, EJS templates, Docker, Docker Compose

---

## Task 1: Project Scaffolding

**Delete all prototype code and create fresh structure.**

**Files:**
- Create: `package.json`
- Create: `.gitignore`
- Delete: All existing prototype files (`app.js`, `controllers/`, `routes/`, `views/`, `public/`, `uploads/`)
- Create: Directory structure
  - `lib/` - Core logic modules
  - `routes/` - Express route handlers
  - `views/` - EJS templates
  - `public/` - Static assets (CSS, JS, images)
  - `templates/` - Docker Compose and OCIS config templates
  - `.env.example` - Example configuration file

**Step 1: Create package.json**
```json
{
  "name": "word-office-opencloud",
  "version": "1.0.0",
  "description": "Deployment companion for Word-Office Document Server + OCIS",
  "main": "app.js",
  "scripts": {
    "start": "node app.js",
    "dev": "nodemon app.js",
    "test": "echo \"Tests not yet implemented\"",
    "lint": "eslint .",
    "lint:fix": "eslint . --fix"
  },
  "keywords": ["ocis", "Word Office", "Word Office", "wopi", "docker"],
  "author": "Word-Office",
  "license": "AGPL-3.0",
  "dependencies": {
    "express": "^4.18.2",
    "ejs": "^3.1.9",
    "dotenv": "^16.3.1",
    "dockerode": "^4.0.0",
    "axios": "^1.6.0",
    "express-rate-limit": "^7.1.5",
    "helmet": "^7.1.0",
    "cors": "^2.8.5"
  },
  "devDependencies": {
    "nodemon": "^3.0.1",
    "eslint": "^8.54.0"
  },
  "engines": {
    "node": ">=20.0.0"
  }
}
```

**Step 2: Create .gitignore**
```
node_modules/
.env
*.log
.DS_Store
uploads/
.docker/
logs/
coverage/
.vscode/
.idea/
```

**Step 3: Create .env.example**
```
# word-office-opencloud Configuration
# Copy this file to .env and fill in your values

# Application
PORT=3000
NODE_ENV=production

# Domains
OCIS_DOMAIN=ocis.example.com
DOCUMENT_SERVER_DOMAIN=docs.example.com

# Secrets (generate with: openssl rand -base64 32)
OCIS_JWT_SECRET=change_me_jwt_secret_minimum_32_characters
DOCUMENT_SERVER_JWT_SECRET=change_me_document_server_jwt_secret

# Docker Configuration
OCIS_IMAGE=owncloud/ocis:latest
DOCUMENT_SERVER_IMAGE=Word Office/documentserver:latest
TRAEFIK_IMAGE=traefik:v2.10

# Ports (adjust if already in use)
TRAEFIK_HTTP_PORT=80
TRAEFIK_HTTPS_PORT=443
OCIS_INTERNAL_PORT=9200
DOCUMENT_SERVER_INTERNAL_PORT=80

# Storage
OCIS_DATA_DIR=./data/ocis
DOCUMENT_SERVER_DATA_DIR=./data/documentserver
TRAEFIK_DATA_DIR=./data/traefik

# Features (true/false)
ENABLE_SSL=true
ENABLE_METRICS=true
ENABLE_LOGS=true
```

**Step 4: Delete prototype files**
- Delete: `app.js`
- Delete: `controllers/` directory
- Delete: `routes/` directory
- Delete: `views/` directory
- Delete: `public/` directory
- Delete: `uploads/` directory

**Step 5: Create directory structure**
```bash
mkdir -p lib routes views public/templates
mkdir -p templates/docker templates/ocis
```

**Verification:**
```bash
# Verify files exist
test -f package.json && echo "package.json exists"
test -f .gitignore && echo ".gitignore exists"
test -f .env.example && echo ".env.example exists"
test -d lib && echo "lib directory exists"
test -d routes && echo "routes directory exists"
test -d views && echo "views directory exists"
test -d public && echo "public directory exists"
test -d templates && echo "templates directory exists"

# Verify prototype files deleted
! test -f app.js && echo "app.js deleted"
```

**Commit:**
```bash
git add .
git commit -m "feat: scaffold project structure, delete prototype code"
```

---

## Task 2: Configuration Module

**Implement configuration loader and validator.**

**Files:**
- Create: `lib/config.js`

**Step 1: Create lib/config.js**
```javascript
const dotenv = require('dotenv');
const path = require('path');

// Load environment variables
dotenv.config();

// Required fields with defaults
const defaults = {
  PORT: '3000',
  NODE_ENV: 'development',
  OCIS_IMAGE: 'owncloud/ocis:latest',
  DOCUMENT_SERVER_IMAGE: 'Word Office/documentserver:latest',
  TRAEFIK_IMAGE: 'traefik:v2.10',
  TRAEFIK_HTTP_PORT: '80',
  TRAEFIK_HTTPS_PORT: '443',
  OCIS_INTERNAL_PORT: '9200',
  DOCUMENT_SERVER_INTERNAL_PORT: '80',
  OCIS_DATA_DIR: './data/ocis',
  DOCUMENT_SERVER_DATA_DIR: './data/documentserver',
  TRAEFIK_DATA_DIR: './data/traefik',
  ENABLE_SSL: 'true',
  ENABLE_METRICS: 'true',
  ENABLE_LOGS: 'true'
};

// Required fields (no defaults)
const required = [
  'OCIS_DOMAIN',
  'DOCUMENT_SERVER_DOMAIN',
  'OCIS_JWT_SECRET',
  'DOCUMENT_SERVER_JWT_SECRET'
];

function loadConfig() {
  // Merge defaults with environment variables
  const config = { ...defaults };

  // Load all environment variables
  Object.keys(process.env).forEach(key => {
    config[key] = process.env[key];
  });

  // Validate required fields
  const missing = required.filter(field => !config[field]);
  if (missing.length > 0) {
    throw new Error(`Missing required configuration: ${missing.join(', ')}`);
  }

  // Validate JWT secrets (minimum 32 characters)
  if (config.OCIS_JWT_SECRET.length < 32) {
    throw new Error('OCIS_JWT_SECRET must be at least 32 characters');
  }
  if (config.DOCUMENT_SERVER_JWT_SECRET.length < 32) {
    throw new Error('DOCUMENT_SERVER_JWT_SECRET must be at least 32 characters');
  }

  // Validate ports are numbers
  const ports = [
    'PORT', 'TRAEFIK_HTTP_PORT', 'TRAEFIK_HTTPS_PORT',
    'OCIS_INTERNAL_PORT', 'DOCUMENT_SERVER_INTERNAL_PORT'
  ];
  ports.forEach(port => {
    if (isNaN(parseInt(config[port]))) {
      throw new Error(`${port} must be a valid number`);
    }
  });

  // Convert boolean strings to booleans
  config.ENABLE_SSL = config.ENABLE_SSL === 'true';
  config.ENABLE_METRICS = config.ENABLE_METRICS === 'true';
  config.ENABLE_LOGS = config.ENABLE_LOGS === 'true';

  return config;
}

// Export configuration object
const config = loadConfig();

// Export derived values for convenience
config.OCIS_WOPI_SRC = `https://${config.OCIS_DOMAIN}`;
config.DOCUMENT_SERVER_INTERNAL_URL = `http://documentserver:${config.DOCUMENT_SERVER_INTERNAL_PORT}`;
config.OCIS_INTERNAL_URL = `http://ocis:${config.OCIS_INTERNAL_PORT}`;
config.COLLABORATION_APP_ADDR = config.DOCUMENT_SERVER_INTERNAL_URL;
config.COLLABORATION_APP_NAME = 'Word-Office Document Server';

// Path helpers
config.resolvePath = (relativePath) => path.resolve(__dirname, '..', relativePath);

module.exports = config;
```

**Step 2: Create test for config validation**
```bash
# Test missing required field
node -e "
process.env.OCIS_DOMAIN = 'test.com';
process.env.DOCUMENT_SERVER_DOMAIN = 'docs.com';
// Missing OCIS_JWT_SECRET and DOCUMENT_SERVER_JWT_SECRET
require('./lib/config.js');
"
```
Expected: Error "Missing required configuration"

**Step 3: Test with valid config**
```bash
node -e "
require('./lib/config.js');
console.log('Configuration loaded successfully');
"
```
Expected: "Configuration loaded successfully"

**Verification:**
```bash
# Test config loads without errors
node -e "require('./lib/config.js'); console.log('PASS: Config loaded')"

# Test validation of JWT secrets
node -e "
process.env.OCIS_JWT_SECRET = 'short';
require('./lib/config.js');
"
```
Expected: Error about JWT_SECRET minimum length

**Commit:**
```bash
git add lib/config.js
git commit -m "feat: add configuration module with validation"
```

---

## Task 3: Docker Compose Generation

**Generate full stack Docker Compose file from configuration.**

**Files:**
- Create: `lib/compose.js`

**Step 1: Create lib/compose.js**
```javascript
const config = require('./config.js');
const fs = require('fs').promises;
const path = require('path');

async function generateDockerCompose() {
  const compose = {
    version: '3.8',
    services: {}
  };

  // Traefik proxy
  compose.services.traefik = {
    image: config.TRAEFIK_IMAGE,
    container_name: 'word-office-traefik',
    ports: [
      `${config.TRAEFIK_HTTP_PORT}:80`,
      `${config.TRAEFIK_HTTPS_PORT}:443`
    ],
    volumes: [
      `${config.TRAEFIK_DATA_DIR}/letsencrypt:/letsencrypt`,
      `${config.TRAEFIK_DATA_DIR}/logs:/var/log/traefik`,
      '/var/run/docker.sock:/var/run/docker.sock:ro'
    ],
    command: [
      '--api.insecure=true',
      '--providers.docker=true',
      '--providers.docker.exposedbydefault=false',
      '--entrypoints.web.address=:80',
      '--entrypoints.websecure.address=:443'
    ],
    labels: {
      'traefik.enable': 'true'
    },
    networks: ['word-office-network']
  };

  // OCIS service
  compose.services.ocis = {
    image: config.OCIS_IMAGE,
    container_name: 'word-office-ocis',
    environment: [
      `OCIS_DOMAIN=${config.OCIS_DOMAIN}`,
      `OCIS_JWT_SECRET=${config.OCIS_JWT_SECRET}`,
      `OCIS_URL=${config.OCIS_WOPI_SRC}`,
      `OCIS_INSECURE=false`,
      `OCIS_LOG_LEVEL=info`,
      `OCIS_LOG_PRETTY=true`,
      `PROXY_TLS=true`,
      `STORAGE_OCIS_DATA_DIR=/var/lib/ocis`,
      `STORAGE_HOME_DRIVER=owncloudxc`,
      `STORAGE_USERS_DRIVER=owncloudxc`,
      `OCIS_REVA_GATEWAY=${config.OCIS_INTERNAL_URL}`,
      `OCIS_FRONTEND_OCS=${config.OCIS_INTERNAL_URL}`,
      `OCIS_EVENTS_GRPC_ADDR=${config.OCIS_INTERNAL_URL}:9143`,
      `OCIS_GRPC_ADDR=${config.OCIS_INTERNAL_URL}:9143`,
      `OCIS_HTTP_ADDR=:${config.OCIS_INTERNAL_PORT}`,
      `GATEWAY_GRPC_ADDR=${config.OCIS_INTERNAL_URL}:9143`,
      `AUTHENTICATION_HANDLERS=basic`,
      `OCIS_EXCLUDE_RUN_SERVICES=idm,storage-gateway,storage-users,storage-home,ocdav`,
      `IDM_CREATE_DEMO_USERS=false`,
      `WEB_UI_CONFIG_FILE=/etc/ocis/web-ui.json`
    ],
    volumes: [
      `${config.OCIS_DATA_DIR}/data:/var/lib/ocis`,
      `${config.OCIS_DATA_DIR}/config:/etc/ocis`,
      `${config.OCIS_DATA_DIR}/logs:/var/log/ocis`
    ],
    expose: [config.OCIS_INTERNAL_PORT],
    labels: {
      'traefik.enable': 'true',
      'traefik.http.routers.ocis.rule': `Host(\`${config.OCIS_DOMAIN}\`)`,
      'traefik.http.routers.ocis.entrypoints': config.ENABLE_SSL ? 'websecure' : 'web',
      'traefik.http.routers.ocis.tls': config.ENABLE_SSL ? 'true' : 'false',
      'traefik.http.services.ocis.loadbalancer.server.port': config.OCIS_INTERNAL_PORT
    },
    depends_on: ['traefik'],
    networks: ['word-office-network']
  };

  // OCIS Collaboration Service (WOPI)
  compose.services['ocis-collaboration'] = {
    image: config.OCIS_IMAGE,
    container_name: 'word-office-ocis-collaboration',
    command: [
      'ocis',
      'collaboration',
      '--log-level=info',
      '--log-pretty=true'
    ],
    environment: [
      `OCIS_DOMAIN=${config.OCIS_DOMAIN}`,
      `OCIS_JWT_SECRET=${config.OCIS_JWT_SECRET}`,
      `COLLABORATION_WOPI_SRC=${config.OCIS_WOPI_SRC}`,
      `COLLABORATION_APP_ADDR=${config.COLLABORATION_APP_ADDR}`,
      `COLLABORATION_APP_NAME=${config.COLLABORATION_APP_NAME}`,
      `OCIS_REVA_GATEWAY=${config.OCIS_INTERNAL_URL}:9143`,
      `GATEWAY_GRPC_ADDR=${config.OCIS_INTERNAL_URL}:9143`,
      `COLLABORATION_EDITORS_ALBUM=`,
      `COLLABORATION_WOPI_SESSION_ALBUM=`
    ],
    volumes: [
      `${config.OCIS_DATA_DIR}/data:/var/lib/ocis`,
      `${config.OCIS_DATA_DIR}/logs:/var/log/ocis`
    ],
    expose: ['9230'],
    labels: {
      'traefik.enable': 'true',
      'traefik.http.routers.collaboration.rule': `Host(\`${config.OCIS_DOMAIN}\`) && PathPrefix(\`/wopi\`)`,
      'traefik.http.routers.collaboration.entrypoints': config.ENABLE_SSL ? 'websecure' : 'web',
      'traefik.http.routers.collaboration.tls': config.ENABLE_SSL ? 'true' : 'false',
      'traefik.http.services.collaboration.loadbalancer.server.port': '9230'
    },
    depends_on: ['ocis'],
    networks: ['word-office-network']
  };

  // Document Server
  compose.services.documentserver = {
    image: config.DOCUMENT_SERVER_IMAGE,
    container_name: 'word-office-documentserver',
    environment: [
      `JWT_SECRET=${config.DOCUMENT_SERVER_JWT_SECRET}`,
      `JWT_HEADER=Authorization`,
      `JWT_IN_BODY=true`
    ],
    volumes: [
      `${config.DOCUMENT_SERVER_DATA_DIR}/data:/var/www/Word Office/Data`,
      `${config.DOCUMENT_SERVER_DATA_DIR}/logs:/var/log/Word Office`,
      `${config.DOCUMENT_SERVER_DATA_DIR}/lib:/var/lib/Word Office`,
      `${config.DOCUMENT_SERVER_DATA_DIR}/db:/var/lib/postgresql`
    ],
    expose: [config.DOCUMENT_SERVER_INTERNAL_PORT],
    labels: {
      'traefik.enable': 'true',
      'traefik.http.routers.documentserver.rule': `Host(\`${config.DOCUMENT_SERVER_DOMAIN}\`)`,
      'traefik.http.routers.documentserver.entrypoints': config.ENABLE_SSL ? 'websecure' : 'web',
      'traefik.http.routers.documentserver.tls': config.ENABLE_SSL ? 'true' : 'false',
      'traefik.http.services.documentserver.loadbalancer.server.port': config.DOCUMENT_SERVER_INTERNAL_PORT
    },
    depends_on: ['ocis'],
    networks: ['word-office-network']
  };

  // Networks
  compose.networks = {
    'word-office-network': {
      driver: 'bridge'
    }
  };

  return compose;
}

async function writeDockerCompose(outputPath) {
  const compose = await generateDockerCompose();
  const yaml = require('js-yaml');
  const yamlString = yaml.dump(compose, {
    lineWidth: 120,
    noRefs: true
  });

  await fs.writeFile(outputPath, yamlString, 'utf-8');
  return yamlString;
}

module.exports = {
  generateDockerCompose,
  writeDockerCompose
};
```

**Step 2: Add js-yaml dependency**
```bash
npm install js-yaml
```

**Step 3: Test generation**
```bash
node -e "
const { writeDockerCompose } = require('./lib/compose.js');
writeDockerCompose('./docker-compose.yml')
  .then(() => console.log('PASS: docker-compose.yml generated'))
  .catch(err => console.error('FAIL:', err.message));
"
```

**Verification:**
```bash
# Verify docker-compose.yml was generated
test -f docker-compose.yml && echo "PASS: docker-compose.yml exists"

# Check content has all services
grep -q "traefik:" docker-compose.yml && echo "PASS: Traefik service present"
grep -q "ocis:" docker-compose.yml && echo "PASS: OCIS service present"
grep -q "documentserver:" docker-compose.yml && echo "PASS: Document Server present"
grep -q "ocis-collaboration:" docker-compose.yml && echo "PASS: Collaboration service present"

# Validate YAML syntax
docker-compose config > /dev/null 2>&1 && echo "PASS: Valid docker-compose.yml"
```

**Commit:**
```bash
git add lib/compose.js package.json docker-compose.yml
git commit -m "feat: add Docker Compose generation module"
```

---

## Task 4: OCIS WOPI Configuration

**Generate OCIS-specific configuration files.**

**Files:**
- Create: `lib/ocis-config.js`

**Step 1: Create lib/ocis-config.js**
```javascript
const config = require('./config.js');
const fs = require('fs').promises;
const path = require('path');

/**
 * Generate OCIS web-ui.json configuration
 */
function generateWebUIConfig() {
  return {
    server: `${config.OCIS_WOPI_SRC}`,
    theme: 'default',
    openIdConnect: {
      metadataUrl: `${config.OCIS_WOPI_SRC}/.well-known/openid-configuration`
    },
    apps: {
      files: {
        enabled: true,
        path: '/files'
      },
      settings: {
        enabled: true,
        path: '/settings'
      }
    },
    editor: {
      mimeTypeHandlers: [
        {
          extension: [
            'odt', 'fodt', 'ott', 'rtf',
            'ods', 'fods', 'ots',
            'odp', 'fodp', 'otp',
            'docx', 'xlsx', 'pptx'
          ],
          handler: {
            url: `${config.OCIS_WOPI_SRC}/wopi`,
            openNewTab: true,
            hasMenuEntry: true
          }
        }
      ]
    },
    options: {
      hideSearchBar: false,
      disablePreviews: false,
      general: {
        displayName: 'Word-Office Cloud',
        slogan: 'Your documents, anywhere',
        hideVersion: false
      }
    }
  };
}

/**
 * Generate OCIS identity provider configuration
 */
function generateIdpConfig() {
  return {
    idp: {
      manager: {
        oidc: {
          issuer: `${config.OCIS_WOPI_SRC}`,
          clientID: 'ocis',
          clientSecret: config.OCIS_JWT_SECRET
        }
      },
      ldap: {},
      basic: {
        enabled: true
      }
    }
  };
}

/**
 * Write all OCIS configuration files
 */
async function writeOcisConfig(outputDir) {
  await fs.mkdir(outputDir, { recursive: true });

  // Write web-ui.json
  const webUIConfig = generateWebUIConfig();
  const webUIPath = path.join(outputDir, 'web-ui.json');
  await fs.writeFile(webUIPath, JSON.stringify(webUIConfig, null, 2), 'utf-8');

  // Write idp.json
  const idpConfig = generateIdpConfig();
  const idpPath = path.join(outputDir, 'idp.json');
  await fs.writeFile(idpPath, JSON.stringify(idpConfig, null, 2), 'utf-8');

  return {
    webUI: webUIPath,
    idp: idpPath
  };
}

/**
 * Generate OCIS environment variable mappings
 */
function getOcisEnvVars() {
  return {
    // Domain and URLs
    OCIS_DOMAIN: config.OCIS_DOMAIN,
    OCIS_URL: config.OCIS_WOPI_SRC,
    OCIS_INSECURE: 'false',

    // JWT
    OCIS_JWT_SECRET: config.OCIS_JWT_SECRET,

    // Services
    OCIS_EXCLUDE_RUN_SERVICES: 'idm,storage-gateway,storage-users,storage-home,ocdav',

    // Storage
    STORAGE_OCIS_DATA_DIR: '/var/lib/ocis',
    STORAGE_HOME_DRIVER: 'owncloudxc',
    STORAGE_USERS_DRIVER: 'owncloudxc',

    // Gateway
    OCIS_REVA_GATEWAY: config.OCIS_INTERNAL_URL,
    OCIS_FRONTEND_OCS: config.OCIS_INTERNAL_URL,
    GATEWAY_GRPC_ADDR: `${config.OCIS_INTERNAL_URL}:9143`,

    // Events
    OCIS_EVENTS_GRPC_ADDR: `${config.OCIS_INTERNAL_URL}:9143`,

    // HTTP
    OCIS_HTTP_ADDR: `:${config.OCIS_INTERNAL_PORT}`,

    // Web UI
    WEB_UI_CONFIG_FILE: '/etc/ocis/web-ui.json',

    // Proxy
    PROXY_TLS: config.ENABLE_SSL ? 'true' : 'false',

    // Auth
    AUTHENTICATION_HANDLERS: 'basic',
    IDM_CREATE_DEMO_USERS: 'false',

    // Logging
    OCIS_LOG_LEVEL: 'info',
    OCIS_LOG_PRETTY: 'true',

    // Collaboration (WOPI)
    COLLABORATION_WOPI_SRC: config.OCIS_WOPI_SRC,
    COLLABORATION_APP_ADDR: config.COLLABORATION_APP_ADDR,
    COLLABORATION_APP_NAME: config.COLLABORATION_APP_NAME
  };
}

module.exports = {
  generateWebUIConfig,
  generateIdpConfig,
  writeOcisConfig,
  getOcisEnvVars
};
```

**Step 2: Test OCIS config generation**
```bash
node -e "
const { writeOcisConfig } = require('./lib/ocis-config.js');
writeOcisConfig('./data/ocis/config')
  .then(paths => console.log('PASS:', paths))
  .catch(err => console.error('FAIL:', err.message));
"
```

**Verification:**
```bash
# Verify OCIS config files generated
test -f data/ocis/config/web-ui.json && echo "PASS: web-ui.json exists"
test -f data/ocis/config/idp.json && echo "PASS: idp.json exists"

# Check web-ui.json has editor config
grep -q "Word-Office Cloud" data/ocis/config/web-ui.json && echo "PASS: Branding present"
grep -q "wopi" data/ocis/config/web-ui.json && echo "PASS: WOPI handler configured"

# Check JSON is valid
node -e "JSON.parse(require('fs').readFileSync('data/ocis/config/web-ui.json'))" && echo "PASS: Valid JSON"
node -e "JSON.parse(require('fs').readFileSync('data/ocis/config/idp.json'))" && echo "PASS: Valid JSON"
```

**Commit:**
```bash
git add lib/ocis-config.js data/ocis/config/
git commit -m "feat: add OCIS WOPI configuration generator"
```

---

## Task 5: Setup Wizard

**Create interactive setup wizard for configuration.**

**Files:**
- Create: `routes/setup.js`
- Create: `views/setup.ejs`

**Step 1: Create routes/setup.js**
```javascript
const express = require('express');
const router = express.Router();
const config = require('../lib/config.js');
const { writeDockerCompose } = require('../lib/compose.js');
const { writeOcisConfig } = require('../lib/ocis-config.js');
const fs = require('fs').promises;
const path = require('path');

/**
 * GET /setup - Display setup wizard
 */
router.get('/', async (req, res) => {
  try {
    // Check if already configured
    const envExists = await fileExists('.env');

    res.render('setup', {
      title: 'word-office-opencloud Setup',
      configured: envExists,
      config: envExists ? config : getDefaultConfig(),
      errors: {}
    });
  } catch (error) {
    res.status(500).render('error', {
      title: 'Error',
      message: error.message
    });
  }
});

/**
 * POST /setup - Save configuration
 */
router.post('/', async (req, res) => {
  try {
    const formData = req.body;
    const errors = validateFormData(formData);

    if (Object.keys(errors).length > 0) {
      return res.render('setup', {
        title: 'word-office-opencloud Setup',
        configured: false,
        config: formData,
        errors
      });
    }

    // Generate secrets if not provided
    const secrets = await generateSecrets();

    // Build .env file content
    const envContent = buildEnvFile(formData, secrets);

    // Write .env file
    await fs.writeFile('.env', envContent, 'utf-8');

    // Generate docker-compose.yml
    await writeDockerCompose('./docker-compose.yml');

    // Generate OCIS config
    await writeOcisConfig('./data/ocis/config');

    res.redirect('/setup?success=true');
  } catch (error) {
    res.status(500).render('error', {
      title: 'Error',
      message: error.message
    });
  }
});

/**
 * Helper functions
 */

async function fileExists(filePath) {
  try {
    await fs.access(filePath);
    return true;
  } catch {
    return false;
  }
}

function getDefaultConfig() {
  return {
    OCIS_DOMAIN: '',
    DOCUMENT_SERVER_DOMAIN: '',
    PORT: '3000',
    ENABLE_SSL: 'true',
    ENABLE_METRICS: 'true',
    ENABLE_LOGS: 'true'
  };
}

function validateFormData(data) {
  const errors = {};

  // Domain validation
  if (!data.OCIS_DOMAIN) {
    errors.OCIS_DOMAIN = 'OCIS domain is required';
  } else if (!isValidDomain(data.OCIS_DOMAIN)) {
    errors.OCIS_DOMAIN = 'Invalid domain format';
  }

  if (!data.DOCUMENT_SERVER_DOMAIN) {
    errors.DOCUMENT_SERVER_DOMAIN = 'Document Server domain is required';
  } else if (!isValidDomain(data.DOCUMENT_SERVER_DOMAIN)) {
    errors.DOCUMENT_SERVER_DOMAIN = 'Invalid domain format';
  }

  // Port validation
  if (data.PORT && !isValidPort(data.PORT)) {
    errors.PORT = 'Invalid port number';
  }

  return errors;
}

function isValidDomain(domain) {
  const domainRegex = /^[a-zA-Z0-9][a-zA-Z0-9-]{0,61}[a-zA-Z0-9](?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$/;
  return domainRegex.test(domain);
}

function isValidPort(port) {
  const portNum = parseInt(port);
  return !isNaN(portNum) && portNum > 0 && portNum < 65536;
}

async function generateSecrets() {
  const crypto = require('crypto');

  return {
    OCIS_JWT_SECRET: crypto.randomBytes(32).toString('base64'),
    DOCUMENT_SERVER_JWT_SECRET: crypto.randomBytes(32).toString('base64')
  };
}

function buildEnvFile(formData, secrets) {
  const env = {
    ...secrets,
    PORT: formData.PORT || '3000',
    NODE_ENV: 'production',
    OCIS_DOMAIN: formData.OCIS_DOMAIN,
    DOCUMENT_SERVER_DOMAIN: formData.DOCUMENT_SERVER_DOMAIN,
    ENABLE_SSL: formData.ENABLE_SSL || 'true',
    ENABLE_METRICS: formData.ENABLE_METRICS || 'true',
    ENABLE_LOGS: formData.ENABLE_LOGS || 'true',
    OCIS_IMAGE: 'owncloud/ocis:latest',
    DOCUMENT_SERVER_IMAGE: 'Word Office/documentserver:latest',
    TRAEFIK_IMAGE: 'traefik:v2.10',
    TRAEFIK_HTTP_PORT: '80',
    TRAEFIK_HTTPS_PORT: '443',
    OCIS_INTERNAL_PORT: '9200',
    DOCUMENT_SERVER_INTERNAL_PORT: '80',
    OCIS_DATA_DIR: './data/ocis',
    DOCUMENT_SERVER_DATA_DIR: './data/documentserver',
    TRAEFIK_DATA_DIR: './data/traefik'
  };

  return Object.entries(env)
    .map(([key, value]) => `${key}=${value}`)
    .join('\n');
}

module.exports = router;
```

**Step 2: Create views/setup.ejs**
```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title><%= title %></title>
  <style>
    * {
      margin: 0;
      padding: 0;
      box-sizing: border-box;
    }

    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      background-color: #0b0b1e;
      color: #ffffff;
      line-height: 1.6;
      min-height: 100vh;
      display: flex;
      flex-direction: column;
    }

    .container {
      max-width: 800px;
      margin: 0 auto;
      padding: 40px 20px;
      flex: 1;
    }

    h1 {
      font-weight: 300;
      font-size: 2.5em;
      margin-bottom: 0.5em;
      letter-spacing: 0.05em;
    }

    .subtitle {
      color: #00d4ff;
      margin-bottom: 40px;
      font-weight: 400;
    }

    .form-group {
      margin-bottom: 25px;
    }

    label {
      display: block;
      margin-bottom: 8px;
      color: #ffffff;
      font-weight: 300;
    }

    input[type="text"],
    input[type="number"] {
      width: 100%;
      padding: 12px 16px;
      background-color: #1a1a2e;
      border: 1px solid #00d4ff;
      border-radius: 4px;
      color: #ffffff;
      font-size: 16px;
      font-family: inherit;
    }

    input[type="text"]:focus,
    input[type="number"]:focus {
      outline: none;
      border-color: #e8b931;
    }

    input[type="text"]::placeholder {
      color: #ffffff 0.5;
    }

    .error {
      color: #ff4757;
      font-size: 14px;
      margin-top: 5px;
    }

    .checkbox-group {
      display: flex;
      gap: 30px;
      margin-top: 10px;
    }

    .checkbox-label {
      display: flex;
      align-items: center;
      cursor: pointer;
    }

    .checkbox-label input {
      margin-right: 8px;
      width: 20px;
      height: 20px;
      accent-color: #00d4ff;
    }

    button[type="submit"] {
      background-color: #00d4ff;
      color: #0b0b1e;
      border: none;
      padding: 14px 32px;
      font-size: 18px;
      font-weight: 600;
      border-radius: 4px;
      cursor: pointer;
      transition: all 0.3s ease;
      letter-spacing: 0.02em;
    }

    button[type="submit"]:hover {
      background-color: #e8b931;
    }

    .success-message {
      background-color: #1a472a;
      border-left: 4px solid #00d4ff;
      padding: 20px;
      margin-bottom: 30px;
      border-radius: 4px;
    }

    .success-message h3 {
      color: #00d4ff;
      margin-bottom: 10px;
    }

    .info-box {
      background-color: #1a1a2e;
      border-left: 4px solid #e8b931;
      padding: 15px 20px;
      margin-bottom: 30px;
      border-radius: 4px;
    }

    .info-box p {
      color: #ffffff 0.9;
    }

    footer {
      text-align: center;
      padding: 20px;
      color: #ffffff 0.5;
      font-size: 14px;
    }
  </style>
</head>
<body>
  <div class="container">
    <h1><%= title %></h1>
    <p class="subtitle">Configure your Word-Office Cloud deployment</p>

    <% if (configured) { %>
      <div class="success-message">
        <h3>Configuration Complete!</h3>
        <p>Your Word-Office-opencloud is configured. You can now start the services.</p>
      </div>

      <div class="info-box">
        <p><strong>Next steps:</strong></p>
        <ul style="margin-left: 20px; margin-top: 10px;">
          <li>Run <code style="background: #0b0b1e; padding: 2px 6px; border-radius: 3px;">docker-compose up -d</code> to start services</li>
          <li>Visit <a href="http://<%= config.OCIS_DOMAIN %>" style="color: #00d4ff;"><%= config.OCIS_DOMAIN %></a> to access OCIS</li>
          <li>Check the <a href="/" style="color: #00d4ff;">dashboard</a> for service status</li>
        </ul>
      </div>
    <% } %>

    <form action="/setup" method="POST">
      <div class="form-group">
        <label for="OCIS_DOMAIN">OCIS Domain</label>
        <input
          type="text"
          id="OCIS_DOMAIN"
          name="OCIS_DOMAIN"
          value="<%= config.OCIS_DOMAIN %>"
          placeholder="ocis.example.com"
          required
        >
        <% if (errors.OCIS_DOMAIN) { %>
          <div class="error"><%= errors.OCIS_DOMAIN %></div>
        <% } %>
      </div>

      <div class="form-group">
        <label for="DOCUMENT_SERVER_DOMAIN">Document Server Domain</label>
        <input
          type="text"
          id="DOCUMENT_SERVER_DOMAIN"
          name="DOCUMENT_SERVER_DOMAIN"
          value="<%= config.DOCUMENT_SERVER_DOMAIN %>"
          placeholder="docs.example.com"
          required
        >
        <% if (errors.DOCUMENT_SERVER_DOMAIN) { %>
          <div class="error"><%= errors.DOCUMENT_SERVER_DOMAIN %></div>
        <% } %>
      </div>

      <div class="form-group">
        <label for="PORT">Dashboard Port</label>
        <input
          type="number"
          id="PORT"
          name="PORT"
          value="<%= config.PORT || '3000' %>"
          placeholder="3000"
        >
        <% if (errors.PORT) { %>
          <div class="error"><%= errors.PORT %></div>
        <% } %>
      </div>

      <div class="form-group">
        <label>Features</label>
        <div class="checkbox-group">
          <label class="checkbox-label">
            <input
              type="checkbox"
              name="ENABLE_SSL"
              value="true"
              <%= config.ENABLE_SSL === 'true' ? 'checked' : '' %>
            >
            Enable SSL/TLS
          </label>
          <label class="checkbox-label">
            <input
              type="checkbox"
              name="ENABLE_METRICS"
              value="true"
              <%= config.ENABLE_METRICS === 'true' ? 'checked' : '' %>
            >
            Enable Metrics
          </label>
          <label class="checkbox-label">
            <input
              type="checkbox"
              name="ENABLE_LOGS"
              value="true"
              <%= config.ENABLE_LOGS === 'true' ? 'checked' : '' %>
            >
            Enable Logs
          </label>
        </div>
      </div>

      <button type="submit">Save Configuration</button>
    </form>
  </div>

  <footer>
    &copy; 2024 Word-Office. Released under AGPL-3.0.
  </footer>
</body>
</html>
```

**Step 3: Create error view**
```html
<!-- views/error.ejs -->
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Error</title>
  <style>
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      background-color: #0b0b1e;
      color: #ffffff;
      display: flex;
      align-items: center;
      justify-content: center;
      min-height: 100vh;
      margin: 0;
    }
    .error-container {
      text-align: center;
      padding: 40px;
    }
    h1 {
      color: #ff4757;
      font-weight: 300;
    }
    .message {
      color: #ffffff 0.9;
      margin: 20px 0;
    }
    a {
      color: #00d4ff;
      text-decoration: none;
    }
    a:hover {
      color: #e8b931;
    }
  </style>
</head>
<body>
  <div class="error-container">
    <h1>Error</h1>
    <p class="message"><%= message %></p>
    <a href="/">Return to Dashboard</a>
  </div>
</body>
</html>
```

**Verification:**
```bash
# Test setup route (will need to start server first)
# Run: npm start
# Visit: http://localhost:3000/setup
# Expected: Setup form displayed
```

**Commit:**
```bash
git add routes/setup.js views/setup.ejs views/error.ejs
git commit -m "feat: add interactive setup wizard"
```

---

## Task 6: Dashboard

**Create main dashboard for service status and management.**

**Files:**
- Create: `routes/dashboard.js`
- Create: `views/dashboard.ejs`

**Step 1: Create routes/dashboard.js**
```javascript
const express = require('express');
const router = express.Router();
const Docker = require('dockerode');
const config = require('../lib/config.js');

const docker = new Docker();

/**
 * GET / - Main dashboard
 */
router.get('/', async (req, res) => {
  try {
    const containers = await getContainers();
    const healthStatus = await getHealthStatus(containers);

    res.render('dashboard', {
      title: 'word-office-opencloud Dashboard',
      config,
      containers,
      healthStatus
    });
  } catch (error) {
    res.status(500).render('error', {
      title: 'Error',
      message: error.message
    });
  }
});

/**
 * GET /api/status - Health status API
 */
router.get('/api/status', async (req, res) => {
  try {
    const containers = await getContainers();
    const healthStatus = await getHealthStatus(containers);

    res.json(healthStatus);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

/**
 * POST /restart/:container - Restart a container
 */
router.post('/restart/:container', async (req, res) => {
  try {
    const containerName = req.params.container;
    const container = docker.getContainer(containerName);

    await container.restart();

    res.json({ success: true, message: `Container ${containerName} restarted` });
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

/**
 * POST /logs/:container - Get container logs
 */
router.post('/logs/:container', async (req, res) => {
  try {
    const containerName = req.params.container;
    const container = docker.getContainer(containerName);

    const logs = await container.logs({
      stdout: true,
      stderr: true,
      tail: 100,
      timestamps: true
    });

    res.json({ logs: logs.toString('utf-8') });
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

/**
 * Helper functions
 */

async function getContainers() {
  const allContainers = await docker.listContainers({ all: true });

  // Filter for our containers
  const ourContainers = allContainers.filter(container => {
    const name = container.Names[0].replace(/^\//, '');
    return name.startsWith('word-office-');
  });

  return ourContainers;
}

async function getHealthStatus(containers) {
  const status = {
    overall: 'unknown',
    services: {}
  };

  const serviceNames = {
    'word-office-traefik': 'Traefik Proxy',
    'word-office-ocis': 'OCIS',
    'word-office-ocis-collaboration': 'OCIS Collaboration (WOPI)',
    'word-office-documentserver': 'Document Server'
  };

  let runningCount = 0;
  let totalCount = Object.keys(serviceNames).length;

  for (const container of containers) {
    const name = container.Names[0].replace(/^\//, '');
    const displayName = serviceNames[name] || name;

    status.services[name] = {
      name: displayName,
      state: container.State,
      status: container.Status,
      running: container.State === 'running',
      image: container.Image,
      ports: container.Ports
    };

    if (container.State === 'running') {
      runningCount++;
    }
  }

  // Determine overall status
  if (runningCount === totalCount) {
    status.overall = 'healthy';
  } else if (runningCount > 0) {
    status.overall = 'degraded';
  } else {
    status.overall = 'down';
  }

  status.running = runningCount;
  status.total = totalCount;

  return status;
}

module.exports = router;
```

**Step 2: Create views/dashboard.ejs**
```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title><%= title %></title>
  <style>
    * {
      margin: 0;
      padding: 0;
      box-sizing: border-box;
    }

    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      background-color: #0b0b1e;
      color: #ffffff;
      line-height: 1.6;
    }

    .header {
      background-color: #1a1a2e;
      padding: 20px 40px;
      border-bottom: 1px solid #00d4ff;
    }

    .header h1 {
      font-weight: 300;
      font-size: 2em;
      letter-spacing: 0.05em;
    }

    .container {
      max-width: 1200px;
      margin: 0 auto;
      padding: 40px 20px;
    }

    .status-banner {
      padding: 20px;
      border-radius: 8px;
      margin-bottom: 30px;
      display: flex;
      align-items: center;
      justify-content: space-between;
    }

    .status-banner.healthy {
      background-color: #1a472a;
      border-left: 4px solid #00d4ff;
    }

    .status-banner.degraded {
      background-color: #1a3a2a;
      border-left: 4px solid #e8b931;
    }

    .status-banner.down {
      background-color: #3a1a1a;
      border-left: 4px solid #ff4757;
    }

    .status-text {
      font-size: 1.2em;
    }

    .services-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
      gap: 20px;
      margin-bottom: 30px;
    }

    .service-card {
      background-color: #1a1a2e;
      border: 1px solid #ffffff 0.1;
      border-radius: 8px;
      padding: 20px;
      transition: all 0.3s ease;
    }

    .service-card:hover {
      border-color: #00d4ff;
    }

    .service-name {
      font-size: 1.3em;
      font-weight: 300;
      margin-bottom: 10px;
      color: #00d4ff;
    }

    .service-status {
      display: flex;
      align-items: center;
      margin-bottom: 15px;
    }

    .status-indicator {
      width: 12px;
      height: 12px;
      border-radius: 50%;
      margin-right: 10px;
    }

    .status-indicator.running {
      background-color: #00d4ff;
      box-shadow: 0 0 10px #00d4ff;
    }

    .status-indicator.exited {
      background-color: #ff4757;
    }

    .status-indicator.paused {
      background-color: #e8b931;
    }

    .service-info {
      font-size: 0.9em;
      color: #ffffff 0.7;
    }

    .action-buttons {
      display: flex;
      gap: 10px;
      margin-top: 15px;
    }

    button {
      background-color: #1a1a2e;
      border: 1px solid #00d4ff;
      color: #00d4ff;
      padding: 8px 16px;
      border-radius: 4px;
      cursor: pointer;
      font-size: 0.9em;
      transition: all 0.3s ease;
    }

    button:hover {
      background-color: #00d4ff;
      color: #0b0b1e;
    }

    .config-section {
      background-color: #1a1a2e;
      border: 1px solid #ffffff 0.1;
      border-radius: 8px;
      padding: 25px;
      margin-bottom: 30px;
    }

    .config-section h2 {
      font-weight: 300;
      font-size: 1.5em;
      margin-bottom: 20px;
      color: #e8b931;
    }

    .config-item {
      display: flex;
      margin-bottom: 15px;
    }

    .config-label {
      width: 200px;
      color: #ffffff 0.7;
    }

    .config-value {
      color: #00d4ff;
      font-weight: 500;
    }

    .quick-actions {
      display: flex;
      gap: 15px;
      margin-bottom: 30px;
    }

    .quick-action {
      background-color: #00d4ff;
      color: #0b0b1e;
      padding: 12px 24px;
      border-radius: 4px;
      text-decoration: none;
      font-weight: 500;
      transition: all 0.3s ease;
    }

    .quick-action:hover {
      background-color: #e8b931;
    }

    .modal {
      display: none;
      position: fixed;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      background-color: #0b0b1e 0.9;
      z-index: 1000;
      justify-content: center;
      align-items: center;
    }

    .modal.active {
      display: flex;
    }

    .modal-content {
      background-color: #1a1a2e;
      border: 1px solid #00d4ff;
      border-radius: 8px;
      padding: 30px;
      max-width: 800px;
      width: 90%;
      max-height: 80vh;
      overflow-y: auto;
    }

    .modal-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 20px;
    }

    .modal-title {
      font-size: 1.5em;
      font-weight: 300;
    }

    .modal-close {
      background: none;
      border: none;
      color: #ffffff;
      font-size: 2em;
      cursor: pointer;
    }

    .logs-content {
      background-color: #0b0b1e;
      border: 1px solid #ffffff 0.1;
      border-radius: 4px;
      padding: 15px;
      font-family: 'Courier New', monospace;
      font-size: 0.9em;
      white-space: pre-wrap;
      word-break: break-all;
    }
  </style>
</head>
<body>
  <div class="header">
    <h1><%= title %></h1>
  </div>

  <div class="container">
    <% if (healthStatus.overall === 'healthy') { %>
      <div class="status-banner healthy">
        <div>
          <strong>All Systems Operational</strong>
          <span style="margin-left: 10px;"><%= healthStatus.running %>/<%= healthStatus.total %> services running</span>
        </div>
      </div>
    <% } else if (healthStatus.overall === 'degraded') { %>
      <div class="status-banner degraded">
        <div>
          <strong>System Degraded</strong>
          <span style="margin-left: 10px;"><%= healthStatus.running %>/<%= healthStatus.total %> services running</span>
        </div>
      </div>
    <% } else { %>
      <div class="status-banner down">
        <div>
          <strong>System Down</strong>
          <span style="margin-left: 10px;">No services running</span>
        </div>
      </div>
    <% } %>

    <div class="quick-actions">
      <a href="/setup" class="quick-action">Run Setup</a>
      <a href="/api/status" class="quick-action" target="_blank">API Status</a>
    </div>

    <div class="services-grid">
      <% Object.values(healthStatus.services).forEach(service => { %>
        <div class="service-card">
          <div class="service-name"><%= service.name %></div>
          <div class="service-status">
            <div class="status-indicator <%= service.running ? 'running' : 'exited' %>"></div>
            <span><%= service.running ? 'Running' : 'Stopped' %></span>
          </div>
          <div class="service-info">
            <div>State: <%= service.state %></div>
            <div><%= service.status %></div>
          </div>
          <div class="action-buttons">
            <button onclick="restartService('<%= Object.keys(healthStatus.services).find(key => healthStatus.services[key].name === service.name) %>')">Restart</button>
            <button onclick="viewLogs('<%= Object.keys(healthStatus.services).find(key => healthStatus.services[key].name === service.name) %>')">Logs</button>
          </div>
        </div>
      <% }); %>
    </div>

    <div class="config-section">
      <h2>Configuration</h2>
      <div class="config-item">
        <div class="config-label">OCIS Domain:</div>
        <div class="config-value"><%= config.OCIS_DOMAIN %></div>
      </div>
      <div class="config-item">
        <div class="config-label">Document Server Domain:</div>
        <div class="config-value"><%= config.DOCUMENT_SERVER_DOMAIN %></div>
      </div>
      <div class="config-item">
        <div class="config-label">Dashboard Port:</div>
        <div class="config-value"><%= config.PORT %></div>
      </div>
      <div class="config-item">
        <div class="config-label">SSL Enabled:</div>
        <div class="config-value"><%= config.ENABLE_SSL ? 'Yes' : 'No' %></div>
      </div>
    </div>
  </div>

  <!-- Logs Modal -->
  <div id="logsModal" class="modal">
    <div class="modal-content">
      <div class="modal-header">
        <h2 class="modal-title">Container Logs</h2>
        <button class="modal-close" onclick="closeModal()">&times;</button>
      </div>
      <div id="logsContent" class="logs-content">Loading logs...</div>
    </div>
  </div>

  <script>
    async function restartService(containerName) {
      if (!confirm(`Restart ${containerName}?`)) return;

      try {
        const response = await fetch(`/restart/${containerName}`, { method: 'POST' });
        const data = await response.json();

        if (data.success) {
          alert(data.message);
          location.reload();
        } else {
          alert('Error: ' + data.error);
        }
      } catch (error) {
        alert('Error: ' + error.message);
      }
    }

    async function viewLogs(containerName) {
      document.getElementById('logsModal').classList.add('active');
      document.getElementById('logsContent').textContent = 'Loading logs...';

      try {
        const response = await fetch(`/logs/${containerName}`, { method: 'POST' });
        const data = await response.json();

        if (data.logs) {
          document.getElementById('logsContent').textContent = data.logs;
        } else {
          document.getElementById('logsContent').textContent = 'Error: ' + data.error;
        }
      } catch (error) {
        document.getElementById('logsContent').textContent = 'Error: ' + error.message;
      }
    }

    function closeModal() {
      document.getElementById('logsModal').classList.remove('active');
    }

    // Close modal on background click
    document.getElementById('logsModal').addEventListener('click', (e) => {
      if (e.target.id === 'logsModal') {
        closeModal();
      }
    });

    // Auto-refresh dashboard every 30 seconds
    setInterval(() => {
      location.reload();
    }, 30000);
  </script>
</body>
</html>
```

**Verification:**
```bash
# Test dashboard (will need to start server first)
# Run: npm start
# Visit: http://localhost:3000/
# Expected: Dashboard displayed with service status
```

**Commit:**
```bash
git add routes/dashboard.js views/dashboard.ejs
git commit -m "feat: add service status dashboard"
```

---

## Task 7: Health Check API

**Add comprehensive health check endpoints.**

**Files:**
- Create: `lib/health.js`
- Modify: `routes/api.js` (create if doesn't exist)

**Step 1: Create lib/health.js**
```javascript
const Docker = require('dockerode');
const axios = require('axios');
const config = require('./config.js');

const docker = new Docker();

/**
 * Check if a container is running
 */
async function isContainerRunning(containerName) {
  try {
    const container = docker.getContainer(containerName);
    const info = await container.inspect();
    return info.State.Running;
  } catch (error) {
    return false;
  }
}

/**
 * Check if a URL is accessible
 */
async function isUrlAccessible(url, timeout = 5000) {
  try {
    const response = await axios.get(url, {
      timeout,
      validateStatus: () => true
    });
    return response.status < 500;
  } catch (error) {
    return false;
  }
}

/**
 * Get container health check result
 */
async function getContainerHealth(containerName, displayName, healthUrl) {
  const running = await isContainerRunning(containerName);
  const health = {
    name: displayName,
    containerName,
    running,
    healthy: false,
    url: healthUrl
  };

  if (running && healthUrl) {
    try {
      health.healthy = await isUrlAccessible(healthUrl);
    } catch (error) {
      health.healthy = false;
    }
  }

  return health;
}

/**
 * Get comprehensive health status
 */
async function getFullHealthStatus() {
  const services = [];

  // Check Traefik
  services.push(await getContainerHealth(
    'word-office-traefik',
    'Traefik Proxy',
    `http://localhost:${config.TRAEFIK_HTTP_PORT}/`
  ));

  // Check OCIS
  services.push(await getContainerHealth(
    'word-office-ocis',
    'OCIS',
    config.ENABLE_SSL
      ? `https://${config.OCIS_DOMAIN}`
      : `http://${config.OCIS_DOMAIN}`
  ));

  // Check OCIS Collaboration (WOPI)
  services.push(await getContainerHealth(
    'word-office-ocis-collaboration',
    'OCIS Collaboration (WOPI)',
    `${config.OCIS_WOPI_SRC}/wopi`
  ));

  // Check Document Server
  services.push(await getContainerHealth(
    'word-office-documentserver',
    'Document Server',
    config.ENABLE_SSL
      ? `https://${config.DOCUMENT_SERVER_DOMAIN}`
      : `http://${config.DOCUMENT_SERVER_DOMAIN}`
  ));

  // Determine overall status
  const runningCount = services.filter(s => s.running).length;
  const healthyCount = services.filter(s => s.healthy).length;

  let overall = 'unknown';
  if (healthyCount === services.length) {
    overall = 'healthy';
  } else if (runningCount > 0) {
    overall = 'degraded';
  } else {
    overall = 'down';
  }

  return {
    overall,
    services,
    running: runningCount,
    healthy: healthyCount,
    total: services.length,
    timestamp: new Date().toISOString()
  };
}

/**
 * Check WOPI connectivity
 */
async function checkWopiConnectivity() {
  try {
    // Check WOPI discovery endpoint
    const discoveryUrl = `${config.OCIS_WOPI_SRC}/wopi/discovery`;
    const response = await axios.get(discoveryUrl, {
      timeout: 5000,
      validateStatus: () => true
    });

    return {
      accessible: response.status < 500,
      statusCode: response.status,
      discoveryUrl
    };
  } catch (error) {
    return {
      accessible: false,
      error: error.message,
      discoveryUrl: `${config.OCIS_WOPI_SRC}/wopi/discovery`
    };
  }
}

/**
 * Get system metrics
 */
async function getSystemMetrics() {
  try {
    const containers = await docker.listContainers({ all: true });
    const ourContainers = containers.filter(c =>
      c.Names[0].startsWith('/word-office-')
    );

    let totalMemory = 0;
    let totalCpu = 0;

    for (const container of ourContainers) {
      if (container.State === 'running') {
        const stats = await docker.getContainer(container.Id).stats({ stream: false });
        totalMemory += stats.memory_stats.usage || 0;
        totalCpu += stats.cpu_stats.cpu_usage.total_usage || 0;
      }
    }

    return {
      containers: ourContainers.length,
      runningContainers: ourContainers.filter(c => c.State === 'running').length,
      memoryUsage: Math.round(totalMemory / 1024 / 1024), // MB
      cpuUsage: totalCpu
    };
  } catch (error) {
    return {
      error: error.message
    };
  }
}

module.exports = {
  isContainerRunning,
  isUrlAccessible,
  getContainerHealth,
  getFullHealthStatus,
  checkWopiConnectivity,
  getSystemMetrics
};
```

**Step 2: Create routes/api.js**
```javascript
const express = require('express');
const router = express.Router();
const health = require('../lib/health.js');

/**
 * GET /api/health - Health check endpoint
 */
router.get('/health', async (req, res) => {
  try {
    const status = await health.getFullHealthStatus();

    res.status(status.overall === 'healthy' ? 200 : 503).json(status);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

/**
 * GET /api/health/wopi - Check WOPI connectivity
 */
router.get('/health/wopi', async (req, res) => {
  try {
    const wopiStatus = await health.checkWopiConnectivity();

    res.status(wopiStatus.accessible ? 200 : 503).json(wopiStatus);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

/**
 * GET /api/metrics - System metrics
 */
router.get('/metrics', async (req, res) => {
  try {
    const metrics = await health.getSystemMetrics();

    res.json(metrics);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

/**
 * GET /api/config - Current configuration (sanitized)
 */
router.get('/config', (req, res) => {
  const config = require('../lib/config.js');

  // Sanitize config - remove secrets
  const sanitized = {
    OCIS_DOMAIN: config.OCIS_DOMAIN,
    DOCUMENT_SERVER_DOMAIN: config.DOCUMENT_SERVER_DOMAIN,
    PORT: config.PORT,
    ENABLE_SSL: config.ENABLE_SSL,
    ENABLE_METRICS: config.ENABLE_METRICS,
    ENABLE_LOGS: config.ENABLE_LOGS
  };

  res.json(sanitized);
});

module.exports = router;
```

**Verification:**
```bash
# Test health endpoints (will need to start server first)
# Run: npm start

# Test health endpoint
curl http://localhost:3000/api/health
# Expected: JSON with overall status

# Test WOPI health
curl http://localhost:3000/api/health/wopi
# Expected: JSON with WOPI connectivity status

# Test metrics
curl http://localhost:3000/api/metrics
# Expected: JSON with system metrics

# Test config
curl http://localhost:3000/api/config
# Expected: JSON with sanitized config
```

**Commit:**
```bash
git add lib/health.js routes/api.js
git commit -m "feat: add health check API endpoints"
```

---

## Task 8: Styling

**Add Word-Office themed CSS and assets.**

**Files:**
- Create: `public/css/style.css`
- Create: `public/css/dashboard.css`
- Create: `public/css/setup.css`

**Step 1: Create public/css/style.css**
```css
/* Word-Office Design Language */

:root {
  /* Colors */
  --bg-void: #0b0b1e;
  --bg-secondary: #1a1a2e;
  --bg-tertiary: #252542;
  --primary: #00d4ff;
  --accent: #e8b931;
  --text-primary: #ffffff;
  --text-secondary: rgba(255, 255, 255, 0.7);
  --text-muted: rgba(255, 255, 255, 0.5);
  --border-color: rgba(255, 255, 255, 0.1);
  --success: #00d4ff;
  --warning: #e8b931;
  --error: #ff4757;

  /* Typography */
  --font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  --font-weight-light: 300;
  --font-weight-normal: 400;
  --font-weight-medium: 500;
  --font-weight-bold: 600;

  /* Spacing */
  --spacing-xs: 0.25rem;
  --spacing-sm: 0.5rem;
  --spacing-md: 1rem;
  --spacing-lg: 1.5rem;
  --spacing-xl: 2rem;
  --spacing-2xl: 3rem;

  /* Border radius */
  --radius-sm: 4px;
  --radius-md: 8px;
  --radius-lg: 12px;

  /* Transitions */
  --transition-fast: 0.15s ease;
  --transition-normal: 0.3s ease;
  --transition-slow: 0.5s ease;
}

/* Reset */
*,
*::before,
*::after {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

html {
  font-size: 16px;
}

body {
  font-family: var(--font-family);
  background-color: var(--bg-void);
  color: var(--text-primary);
  line-height: 1.6;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

/* Typography */
h1, h2, h3, h4, h5, h6 {
  font-weight: var(--font-weight-light);
  letter-spacing: 0.05em;
  margin-bottom: var(--spacing-md);
}

h1 {
  font-size: 2.5rem;
}

h2 {
  font-size: 2rem;
}

h3 {
  font-size: 1.5rem;
}

p {
  margin-bottom: var(--spacing-md);
  color: var(--text-secondary);
}

a {
  color: var(--primary);
  text-decoration: none;
  transition: color var(--transition-fast);
}

a:hover {
  color: var(--accent);
}

/* Buttons */
.btn {
  display: inline-block;
  padding: 0.75rem 1.5rem;
  background-color: var(--primary);
  color: var(--bg-void);
  border: none;
  border-radius: var(--radius-sm);
  font-family: var(--font-family);
  font-size: 1rem;
  font-weight: var(--font-weight-bold);
  text-align: center;
  text-decoration: none;
  cursor: pointer;
  transition: all var(--transition-normal);
  letter-spacing: 0.02em;
}

.btn:hover {
  background-color: var(--accent);
  transform: translateY(-1px);
}

.btn:active {
  transform: translateY(0);
}

.btn-outline {
  background-color: transparent;
  color: var(--primary);
  border: 1px solid var(--primary);
}

.btn-outline:hover {
  background-color: var(--primary);
  color: var(--bg-void);
}

.btn-sm {
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
}

.btn-lg {
  padding: 1rem 2rem;
  font-size: 1.125rem;
}

/* Forms */
input[type="text"],
input[type="email"],
input[type="password"],
input[type="number"],
textarea,
select {
  width: 100%;
  padding: 0.75rem 1rem;
  background-color: var(--bg-secondary);
  border: 1px solid var(--primary);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-family: var(--font-family);
  font-size: 1rem;
  line-height: 1.5;
  transition: border-color var(--transition-fast);
}

input:focus,
textarea:focus,
select:focus {
  outline: none;
  border-color: var(--accent);
}

::placeholder {
  color: var(--text-muted);
}

/* Utility classes */
.text-primary { color: var(--primary); }
.text-accent { color: var(--accent); }
.text-secondary { color: var(--text-secondary); }
.text-muted { color: var(--text-muted); }

.bg-void { background-color: var(--bg-void); }
.bg-secondary { background-color: var(--bg-secondary); }
.bg-tertiary { background-color: var(--bg-tertiary); }

.mt-sm { margin-top: var(--spacing-sm); }
.mt-md { margin-top: var(--spacing-md); }
.mt-lg { margin-top: var(--spacing-lg); }
.mt-xl { margin-top: var(--spacing-xl); }

.mb-sm { margin-bottom: var(--spacing-sm); }
.mb-md { margin-bottom: var(--spacing-md); }
.mb-lg { margin-bottom: var(--spacing-lg); }
.mb-xl { margin-bottom: var(--spacing-xl); }

.p-sm { padding: var(--spacing-sm); }
.p-md { padding: var(--spacing-md); }
.p-lg { padding: var(--spacing-lg); }
.p-xl { padding: var(--spacing-xl); }
```

**Step 2: Create public/css/dashboard.css**
```css
/* Dashboard-specific styles */

.dashboard-header {
  background-color: var(--bg-secondary);
  padding: var(--spacing-xl) var(--spacing-2xl);
  border-bottom: 1px solid var(--primary);
}

.dashboard-title {
  font-size: 2.5rem;
  font-weight: var(--font-weight-light);
  letter-spacing: 0.05em;
}

.status-banner {
  padding: var(--spacing-md) var(--spacing-lg);
  border-radius: var(--radius-md);
  margin-bottom: var(--spacing-xl);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.status-banner.healthy {
  background-color: #1a472a;
  border-left: 4px solid var(--success);
}

.status-banner.degraded {
  background-color: #1a3a2a;
  border-left: 4px solid var(--warning);
}

.status-banner.down {
  background-color: #3a1a1a;
  border-left: 4px solid var(--error);
}

.status-text {
  font-size: 1.25rem;
  font-weight: var(--font-weight-normal);
}

.services-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: var(--spacing-lg);
  margin-bottom: var(--spacing-xl);
}

.service-card {
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  padding: var(--spacing-lg);
  transition: all var(--transition-normal);
}

.service-card:hover {
  border-color: var(--primary);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 212, 255, 0.1);
}

.service-name {
  font-size: 1.5rem;
  font-weight: var(--font-weight-light);
  margin-bottom: var(--spacing-md);
  color: var(--primary);
  letter-spacing: 0.05em;
}

.service-status {
  display: flex;
  align-items: center;
  margin-bottom: var(--spacing-md);
}

.status-indicator {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  margin-right: var(--spacing-sm);
}

.status-indicator.running {
  background-color: var(--success);
  box-shadow: 0 0 8px var(--success);
}

.status-indicator.exited {
  background-color: var(--error);
}

.status-indicator.paused {
  background-color: var(--warning);
}

.service-info {
  font-size: 0.875rem;
  color: var(--text-secondary);
  margin-bottom: var(--spacing-md);
}

.service-info div {
  margin-bottom: var(--spacing-xs);
}

.action-buttons {
  display: flex;
  gap: var(--spacing-sm);
}

.action-buttons button {
  background-color: var(--bg-tertiary);
  border: 1px solid var(--primary);
  color: var(--primary);
  padding: 0.5rem 1rem;
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-size: 0.875rem;
  transition: all var(--transition-normal);
}

.action-buttons button:hover {
  background-color: var(--primary);
  color: var(--bg-void);
}

.config-section {
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  padding: var(--spacing-xl);
}

.config-section h2 {
  font-weight: var(--font-weight-light);
  font-size: 1.75rem;
  margin-bottom: var(--spacing-lg);
  color: var(--accent);
}

.config-item {
  display: flex;
  margin-bottom: var(--spacing-md);
}

.config-label {
  width: 200px;
  color: var(--text-secondary);
  font-weight: var(--font-weight-light);
}

.config-value {
  color: var(--primary);
  font-weight: var(--font-weight-medium);
}

.quick-actions {
  display: flex;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-xl);
  flex-wrap: wrap;
}

.quick-action {
  background-color: var(--primary);
  color: var(--bg-void);
  padding: 0.75rem 1.5rem;
  border-radius: var(--radius-sm);
  text-decoration: none;
  font-weight: var(--font-weight-medium);
  transition: all var(--transition-normal);
}

.quick-action:hover {
  background-color: var(--accent);
  transform: translateY(-1px);
}
```

**Step 3: Create public/css/setup.css**
```css
/* Setup wizard specific styles */

.setup-container {
  max-width: 800px;
  margin: 0 auto;
  padding: var(--spacing-2xl) var(--spacing-lg);
}

.setup-title {
  font-size: 2.5rem;
  font-weight: var(--font-weight-light);
  margin-bottom: var(--spacing-sm);
  letter-spacing: 0.05em;
}

.setup-subtitle {
  color: var(--primary);
  margin-bottom: var(--spacing-2xl);
  font-weight: var(--font-weight-normal);
  font-size: 1.125rem;
}

.form-section {
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  padding: var(--spacing-xl);
  margin-bottom: var(--spacing-lg);
}

.form-group {
  margin-bottom: var(--spacing-lg);
}

.form-group label {
  display: block;
  margin-bottom: var(--spacing-sm);
  color: var(--text-primary);
  font-weight: var(--font-weight-light);
  font-size: 1rem;
}

.form-group input[type="text"],
.form-group input[type="number"] {
  width: 100%;
  padding: 0.75rem 1rem;
  background-color: var(--bg-void);
  border: 1px solid var(--primary);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 1rem;
  font-family: var(--font-family);
  line-height: 1.5;
}

.form-group input:focus {
  outline: none;
  border-color: var(--accent);
}

.form-group input::placeholder {
  color: var(--text-muted);
}

.form-error {
  color: var(--error);
  font-size: 0.875rem;
  margin-top: var(--spacing-xs);
}

.checkbox-group {
  display: flex;
  gap: var(--spacing-lg);
  margin-top: var(--spacing-sm);
  flex-wrap: wrap;
}

.checkbox-label {
  display: flex;
  align-items: center;
  cursor: pointer;
  color: var(--text-secondary);
  font-size: 0.875rem;
}

.checkbox-label input {
  margin-right: var(--spacing-sm);
  width: 18px;
  height: 18px;
  accent-color: var(--primary);
}

.success-message {
  background-color: #1a472a;
  border-left: 4px solid var(--primary);
  padding: var(--spacing-md) var(--spacing-lg);
  margin-bottom: var(--spacing-xl);
  border-radius: var(--radius-sm);
}

.success-message h3 {
  color: var(--primary);
  margin-bottom: var(--spacing-sm);
  font-size: 1.25rem;
}

.info-box {
  background-color: var(--bg-secondary);
  border-left: 4px solid var(--accent);
  padding: var(--spacing-md) var(--spacing-lg);
  margin-bottom: var(--spacing-xl);
  border-radius: var(--radius-sm);
}

.info-box p {
  color: var(--text-secondary);
  margin-bottom: var(--spacing-sm);
}

.info-box code {
  background-color: var(--bg-void);
  padding: 0.125rem 0.375rem;
  border-radius: 3px;
  color: var(--primary);
  font-family: 'Courier New', monospace;
  font-size: 0.875rem;
}

.info-box ul {
  margin-left: var(--spacing-lg);
  margin-top: var(--spacing-md);
}

.info-box li {
  color: var(--text-secondary);
  margin-bottom: var(--spacing-xs);
}
```

**Verification:**
```bash
# Verify CSS files exist
test -f public/css/style.css && echo "PASS: style.css exists"
test -f public/css/dashboard.css && echo "PASS: dashboard.css exists"
test -f public/css/setup.css && echo "PASS: setup.css exists"

# Check for Word-Office color palette
grep -q "#0b0b1e" public/css/style.css && echo "PASS: Deep void background present"
grep -q "#00d4ff" public/css/style.css && echo "PASS: Electric cyan present"
grep -q "#e8b931" public/css/style.css && echo "PASS: Warm gold accent present"
```

**Commit:**
```bash
git add public/css/
git commit -m "feat: add Word-Office themed CSS styles"
```

---

## Task 9: Main App Assembly

**Wire all routes, middleware, and error handling.**

**Files:**
- Create: `app.js`

**Step 1: Create app.js**
```javascript
const express = require('express');
const path = require('path');
const helmet = require('helmet');
const cors = require('cors');
const rateLimit = require('express-rate-limit');
const config = require('./lib/config.js');

// Import routes
const setupRoutes = require('./routes/setup.js');
const dashboardRoutes = require('./routes/dashboard.js');
const apiRoutes = require('./routes/api.js');

// Create Express app
const app = express();

// Security middleware
app.use(helmet({
  contentSecurityPolicy: false, // Allow inline scripts for EJS templates
  crossOriginEmbedderPolicy: false
}));

app.use(cors({
  origin: true,
  credentials: true
}));

// Rate limiting
const limiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100, // 100 requests per window
  message: 'Too many requests from this IP'
});

app.use('/api/', limiter);

// Body parsing
app.use(express.json());
app.use(express.urlencoded({ extended: true }));

// Static files
app.use(express.static(path.join(__dirname, 'public')));

// View engine setup
app.set('views', path.join(__dirname, 'views'));
app.set('view engine', 'ejs');

// Routes
app.use('/setup', setupRoutes);
app.use('/', dashboardRoutes);
app.use('/api', apiRoutes);

// 404 handler
app.use((req, res) => {
  res.status(404).render('error', {
    title: '404 - Not Found',
    message: 'The page you are looking for does not exist.'
  });
});

// Global error handler
app.use((err, req, res, next) => {
  console.error('Error:', err);

  res.status(err.status || 500).render('error', {
    title: 'Error',
    message: err.message || 'An unexpected error occurred.'
  });
});

// Start server
const PORT = config.PORT;
app.listen(PORT, () => {
  console.log(`word-office-opencloud dashboard running on port ${PORT}`);
  console.log(`Visit http://localhost:${PORT} to access the dashboard`);
});

// Graceful shutdown
process.on('SIGTERM', () => {
  console.log('SIGTERM signal received: closing HTTP server');
  process.exit(0);
});

process.on('SIGINT', () => {
  console.log('SIGINT signal received: closing HTTP server');
  process.exit(0);
});

module.exports = app;
```

**Step 2: Test server startup**
```bash
node app.js
```
Expected: Server starts and prints connection message

**Verification:**
```bash
# Test app.js syntax
node -c app.js && echo "PASS: app.js syntax valid"

# Test server starts
timeout 5 node app.js 2>&1 | grep "dashboard running" && echo "PASS: Server starts successfully"

# Test routes exist
grep -q "app.use('/setup'" app.js && echo "PASS: Setup route mounted"
grep -q "app.use('/api'" app.js && echo "PASS: API route mounted"
```

**Commit:**
```bash
git add app.js
git commit -m "feat: assemble main application with all routes"
```

---

## Task 10: Dockerfile

**Create multi-stage Dockerfile for the companion itself.**

**Files:**
- Create: `Dockerfile`
- Create: `.dockerignore`

**Step 1: Create Dockerfile**
```dockerfile
# Stage 1: Build
FROM node:20-alpine AS builder

WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies
RUN npm ci --only=production && \
    npm cache clean --force

# Copy application code
COPY . .

# Verify config exists
RUN test -f lib/config.js || exit 1

# Stage 2: Runtime
FROM node:20-alpine

# Install dumb-init for proper signal handling
RUN apk add --no-cache dumb-init

# Create non-root user
RUN addgroup -g 1001 -S word-office && \
    adduser -S -u 1001 -G word-office word-office

WORKDIR /app

# Copy dependencies and app from builder
COPY --from=builder --chown=word-office:word-office /app/node_modules ./node_modules
COPY --from=builder --chown=word-office:word-office /app ./

# Create data directories
RUN mkdir -p data/ocis data/documentserver data/traefik && \
    chown -R word-office:word-office data

# Switch to non-root user
USER word-office

# Expose dashboard port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD node -e "require('http').get('http://localhost:3000/api/health', (r) => { process.exit(r.statusCode === 200 ? 0 : 1) })"

# Use dumb-init to handle signals properly
ENTRYPOINT ["dumb-init", "--"]

# Start the app
CMD ["node", "app.js"]
```

**Step 2: Create .dockerignore**
```
node_modules
npm-debug.log
.env
.DS_Store
uploads
.docker
logs
coverage
.vscode
.idea
*.md
.git
.gitignore
data/
templates/
```

**Step 3: Test Docker build**
```bash
docker build -t word-office-opencloud:latest .
```
Expected: Image builds successfully

**Verification:**
```bash
# Verify Dockerfile syntax
docker build --no-cache -t test-build . && echo "PASS: Docker builds successfully"

# Verify .dockerignore exists
test -f .dockerignore && echo "PASS: .dockerignore exists"

# Check .dockerignore excludes unnecessary files
grep -q "node_modules" .dockerignore && echo "PASS: node_modules excluded"
grep -q ".env" .dockerignore && echo "PASS: .env excluded"
grep -q "data/" .dockerignore && echo "PASS: data directory excluded"
```

**Commit:**
```bash
git add Dockerfile .dockerignore
git commit -m "feat: add multi-stage Dockerfile"
```

---

## Task 11: README Rewrite

**Update README to reflect new purpose and provide quick start guide.**

**Files:**
- Create: `README.md`
- Delete: Any existing README.md (if different)

**Step 1: Create README.md**
```markdown
# word-office-opencloud

A deployment companion for Word-Office Document Server and ownCloud Infinite Scale (OCIS). It provides a simple way to orchestrate OCIS, Document Server, and supporting services via Docker Compose.

## Features

- **Interactive Setup Wizard**: Web-based configuration with validation
- **Docker Compose Generation**: Automatically generates full stack configuration
- **Health Dashboard**: Real-time service status monitoring
- **WOPI Integration**: Seamless OCIS collaboration with Document Server
- **Word-Office Branding**: Consistent visual identity with deep void theme

## Architecture

word-office-opencloud is a deployment companion that:

1. Generates OCIS and Document Server configurations from a single `.env` file
2. Creates Docker Compose files for the full stack (Traefik, OCIS, Document Server)
3. Provides a web dashboard for monitoring service health
4. Offers a setup wizard for easy initial configuration

The architecture leverages OCIS's native WOPI collaboration service, so no custom protocol code is needed.

## Quick Start

### Prerequisites

- Docker 20.10+ and Docker Compose 2.0+
- Node.js 20+ (for running the companion)
- Two domain names (one for OCIS, one for Document Server)

### Installation

```bash
# Clone the repository
git clone https://codeberg.org/Word-Office/word-office-opencloud.git
cd word-office-opencloud

# Install dependencies
npm install

# Start the companion
npm start
```

### Setup

1. Open your browser and navigate to `http://localhost:3000/setup`
2. Fill in your configuration:
   - OCIS Domain (e.g., `ocis.example.com`)
   - Document Server Domain (e.g., `docs.example.com`)
   - Dashboard Port (default: 3000)
   - Features (SSL, metrics, logs)
3. Click "Save Configuration"
4. Run `docker-compose up -d` to start all services
5. Visit your OCIS domain to start using Word-Office Cloud

### Accessing Services

- **Dashboard**: `http://localhost:3000`
- **OCIS**: `https://<your-ocis-domain>`
- **Document Server**: `https://<your-docs-domain>`
- **Health API**: `http://localhost:3000/api/health`

## Configuration

The application uses a single `.env` file for configuration. After running the setup wizard, the file will be automatically created.

### Required Settings

- `OCIS_DOMAIN`: Public URL for OCIS (e.g., `ocis.example.com`)
- `DOCUMENT_SERVER_DOMAIN`: Public URL for Document Server (e.g., `docs.example.com`)
- `OCIS_JWT_SECRET`: Shared secret for WOPI authentication (min 32 chars)
- `DOCUMENT_SERVER_JWT_SECRET`: JWT secret for Document Server (min 32 chars)

### Optional Settings

- `PORT`: Dashboard port (default: 3000)
- `ENABLE_SSL`: Enable SSL/TLS (default: true)
- `ENABLE_METRICS`: Enable metrics collection (default: true)
- `ENABLE_LOGS`: Enable log collection (default: true)

### Docker Settings

- `OCIS_IMAGE`: OCIS Docker image (default: `owncloud/ocis:latest`)
- `DOCUMENT_SERVER_IMAGE`: Document Server image (default: `Word Office/documentserver:latest`)
- `TRAEFIK_IMAGE`: Traefik image (default: `traefik:v2.10`)

## Development

```bash
# Install development dependencies
npm install

# Run with auto-reload
npm run dev

# Lint code
npm run lint

# Fix linting issues
npm run lint:fix
```

## Docker Deployment

### Using Docker Compose

```bash
# Build the companion image
docker build -t word-office-opencloud:latest .

# Run with Docker Compose
docker-compose up -d
```

### Standalone Docker

```bash
docker run -d \
  --name word-office-opencloud \
  -p 3000:3000 \
  -v $(pwd)/.env:/app/.env \
  -v $(pwd)/data:/app/data \
  word-office-opencloud:latest
```

## API Endpoints

### Health Check

```bash
# Get overall health status
curl http://localhost:3000/api/health

# Check WOPI connectivity
curl http://localhost:3000/api/health/wopi

# Get system metrics
curl http://localhost:3000/api/metrics
```

### Configuration

```bash
# Get current configuration (sanitized)
curl http://localhost:3000/api/config
```

## Services

The Docker Compose stack includes:

- **Traefik**: Reverse proxy with automatic SSL
- **OCIS**: ownCloud Infinite Scale (file sharing platform)
- **OCIS Collaboration**: WOPI service for document editing
- **Document Server**: Word-Office document editors

## Troubleshooting

### Services Not Starting

Check container logs:
```bash
docker-compose logs -f <service-name>
```

### WOPI Connection Issues

Verify JWT secrets match between OCIS and Document Server:
```bash
# Check OCIS JWT secret
docker exec word-office-ocis env | grep JWT_SECRET

# Check Document Server JWT secret
docker exec word-office-documentserver env | grep JWT_SECRET
```

### Dashboard Not Accessible

Ensure the companion is running:
```bash
docker ps | grep word-office-opencloud
```

Check port configuration:
```bash
docker logs word-office-opencloud | grep "port"
```

## Security Considerations

- Never commit `.env` files to version control
- Use strong, randomly generated JWT secrets (minimum 32 characters)
- Enable SSL/TLS in production
- Keep Docker images updated
- Review Traefik security headers

## Architecture Details

### WOPI Protocol

The integration uses the WOPI (Web Application Open Platform Interface) protocol:

- **WOPI Host**: OCIS (handles file storage and user authentication)
- **WOPI Client**: Document Server (provides document editing interface)
- **Authentication**: JWT tokens signed with shared secret

### Data Flow

1. User accesses OCIS web interface
2. User opens a document
3. OCIS WOPI service requests editing session from Document Server
4. Document Server loads in iframe with JWT token
5. User edits, changes sync via WOPI protocol

## Contributing

Contributions are welcome! Please follow these guidelines:

- Fork the repository
- Create a feature branch
- Make your changes
- Add tests if applicable
- Submit a pull request

## License

This project is licensed under the AGPL-3.0 License. See LICENSE file for details.

## Support

- **Documentation**: See `docs/DESIGN.md` for architecture details
- **Issues**: Report bugs on Codeberg
- **Community**: Join our discussions for questions

## Related Projects

- [word-office-nextcloud](https://codeberg.org/Word-Office/word-office-nextcloud) - Nextcloud integration
- [OCIS](https://github.com/owncloud/ocis) - ownCloud Infinite Scale
- [Word Office Document Server](https://github.com/Word Office/DocumentServer) - Document editor

---

**© 2024 Word-Office. Released under AGPL-3.0.**
