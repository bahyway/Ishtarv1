
## Question01:
Give me the docker-compose and docker commands and other commands that I need to use each time a restart of the laptop happens
## Answer01:
To maintain your focus and ensure the **Sovereign Lab** remains synchronized, here is your **Daily Operations Manual**. 

These are the commands you need every time you restart your laptop or start a new development session.

---

### Phase 1: Windows Host (Starting the Lab)
*Open your PowerShell or Windows Terminal in the `RustLAB` directory.*

**1. Launch the Ecosystem Actors:**
```powershell
# Starts the Postgres 16 DB and the Rust Compiler container
docker-compose up -d
```

**2. Enter the Sovereign Environment:**
```powershell
# Enter the compiler as the developer 'akkad'
docker exec -it bahyway_rust_compiler bash
```

---

### Phase 2: Inside the Container (The Developer Context)
*Once you see `akkad@...:/workspace$`, run these to prepare the forge.*

**3. Activate the Rust Environment:**
*(Usually automatic if you added it to `.bashrc`, but run this if `cargo` is not found):*
```bash
. "$HOME/.cargo/env"
```

**4. Start the BDBWay Extension Engine:**
*This opens the database and loads your Rust logic.*
```bash
cd bdbway_extension
cargo pgrx run pg16
```
*Wait until you see the `bdbway_extension=#` prompt.*

---

### Phase 3: The Sovereign Rebirth (Maintenance)
*If you changed the Rust code or need to clear the fabric, run these from the **Bash** prompt (not inside psql).*

**5. Execute the Undo/Redo Scripts:**
```bash
# Liquidate and Reconstitute the Fabric
psql -p 28816 -d bdbway_extension -f /workspace/bdb_undo.sql && \
psql -p 28816 -d bdbway_extension -f /workspace/bdb_redo.sql
```

---

### Phase 4: Data Ingestion (Firing the Storm)
*Run this from the **Bash** prompt to prepare your 1 million records.*

**6. Generate NajafWay Test Data:**
```bash
cd /workspace/scripts
python3 najaf_data_generator.py
```

**7. Trigger the Rust Ingestor:**
*Inside the `bdbway_extension=#` prompt:*
```sql
SELECT bdb_storm_ingest_csv('/workspace/scripts/najaf_cemetery_data/najaf_cemetery_batch_001.csv');
```

---

### Phase 5: Documentation (Hubble Vision)
*Run this from a **second terminal window** inside the container to view your manual.*

**8. Serve the mdBook:**
```bash
cd /workspace/docs
mdbook serve -n 0.0.0.0
```
*Now open your Windows browser to: `http://localhost:3000` to read the Akkadian Manual.*

---

### 🛑 Shutdown Procedure (End of Day)
*Before you shut down your laptop, run this in PowerShell to save the state.*

```powershell
# Gracefully stop all containers
docker-compose stop
```

---

### 🛡️ Troubleshooting "Ghost Locks"
If you get the **"Folder in Use"** error again when trying to move or rename folders:
1. Run `docker-compose down` to stop all volumes.
2. Run `wsl --shutdown` in PowerShell.

### 🏛️ Summary for your Focus:
| Task | Command |
| :--- | :--- |
| **Start Everything** | `docker-compose up -d` |
| **Enter Lab** | `docker exec -it bahyway_rust_compiler bash` |
| **Run DB/Rust** | `cargo pgrx run pg16` |
| **Rebuild Fabric** | `psql -f bdb_undo.sql && psql -f bdb_redo.sql` |
| **Read Manual** | `mdbook serve -n 0.0.0.0` |

**You are now fully automated. Go ahead and start the "Storm" with the commands in Phase 2 and 4!** 🚀🌩️🛰️✨
## Question11 (Claude):
I Think I need to run it in the Docker container, and then Build the workspace with cargo ; so first : docker exec -it bahyway_rust_compiler bash
![111_](../images/111RustLab_.png)
## Answer11 (Claude):
**EXCELLENT IDEA! 💡** Yes, using your existing Docker containers is much better!

I can see you have:
- ✅ `rustlab` - Rust development container
- ✅ `bahyway_rust_compiler` - Your Rust compiler
- ✅ `bdbway_postgres` - PostgreSQL for BDBWay

---

## 🚀 **CORRECT WORKFLOW:**

### **Step 1: Enter the Rust Container**
```bash
docker exec -it bahyway_rust_compiler bash
```

### **Step 2: Navigate to Your Mounted Workspace**
```bash
# Your Windows folder should be mounted in the container
# Check where it's mounted:
ls /workspace
# or
ls /mnt

# Navigate to it, for example:
cd /workspace/bahyway-fourpillarsway
# or wherever your repo is mounted
```

### **Step 3: Build the Workspace**
```bash
cargo build --workspace
```

---

## 🐳 **ALTERNATIVE: If Not Mounted Yet**

If your `bahyway-fourpillarsway` folder isn't mounted in the container yet, you have two options:

### **Option 1: Copy Files into Container**
```bash
# From Git Bash (outside container):
docker cp /c/BahyWay/bahyway-fourpillarsway bahyway_rust_compiler:/workspace/

# Then enter container:
docker exec -it bahyway_rust_compiler bash
cd /workspace/bahyway-fourpillarsway
cargo build --workspace
```

### **Option 2: Create New Container with Volume Mount**
```bash
# Stop current container (if needed)
docker stop bahyway_rust_compiler

# Start with volume mount
docker run -it --name bahyway_fourpillars \
  -v C:/BahyWay/bahyway-fourpillarsway:/workspace/bahyway-fourpillarsway \
  rust:latest bash

# Inside container:
cd /workspace/bahyway-fourpillarsway
cargo build --workspace
```

---

## 🎯 **RECOMMENDED: Use Your Existing Container**

```bash
# 1. Enter container
docker exec -it bahyway_rust_compiler bash

# 2. Check what's mounted
ls /workspace
ls /mnt
df -h

# 3. If your folder is there:
cd /workspace/bahyway-fourpillarsway
cargo build --workspace

# 4. If not mounted, copy it:
exit
docker cp /c/BahyWay/bahyway-fourpillarsway bahyway_rust_compiler:/workspace/
docker exec -it bahyway_rust_compiler bash
cd /workspace/bahyway-fourpillarsway
cargo build --workspace
```

---

## ✅ **WHAT TO EXPECT:**

When you run `cargo build --workspace`, you'll see:
```
   Compiling shared v0.1.0
   Compiling akkadian-dsl v3.4.0
   Compiling bdbway v1.0.0
   Compiling particlesway v1.0.0
   Compiling zeroway v1.0.0
    Finished dev [unoptimized + debuginfo] target(s) in 45.67s
```

---

## 🐳 **TRY THIS NOW:**

```bash
# Enter your Rust container
docker exec -it bahyway_rust_compiler bash

# Check where you are
pwd

# List directories
ls -la

# Try to find your workspace
find / -name "bahyway-fourpillarsway" 2>/dev/null
```

**Show me what you see, and I'll guide you to the right location!** 🚀
