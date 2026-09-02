import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

console.log("==========================================================================");
console.log("🧪 HYPERSPACEDB-SKILLS: SPECIFICATION & VALIDITY TEST SUITE");
console.log("==========================================================================\n");

let passed = 0;
let failed = 0;

function pass(label) {
  console.log(`   ✅ ${label}`);
  passed++;
}

function fail(label, detail) {
  console.error(`   ❌ ${label}${detail ? `: ${detail}` : ""}`);
  failed++;
}

// ── 1. Validate All 6 Skills ────────────────────────────────────────────────
console.log("--------------------------------------------------------------------------");
console.log("📚 STEP 1: Verifying Skills Catalog & YAML Frontmatter");
console.log("--------------------------------------------------------------------------");

const expectedSkills = [
  "hyperspacedb-core",
  "hyperspacedb-graph",
  "hyperspacedb-cognitive",
  "hyperspacedb-memory",
  "hyperspacedb-depin",
  "hyperspacedb-mcp"
];

for (const skill of expectedSkills) {
  const skillPath = path.join(__dirname, "skills", skill, "SKILL.md");
  if (!fs.existsSync(skillPath)) {
    fail(`Missing SKILL.md for ${skill}`);
    continue;
  }
  const content = fs.readFileSync(skillPath, "utf-8");
  const hasFrontmatter = content.startsWith("---") && content.indexOf("---", 3) > 3;
  if (!hasFrontmatter) {
    fail(`Invalid frontmatter in ${skill}`);
    continue;
  }
  const fm = content.substring(3, content.indexOf("---", 3));
  if (fm.includes("name:") && fm.includes("description:")) {
    pass(`Skill ${skill} validated (YAML frontmatter + description)`);
  } else {
    fail(`Frontmatter missing required fields in ${skill}`);
  }
}

// ── 2. Validate Agent Instruction Templates ────────────────────────────────
console.log("\n--------------------------------------------------------------------------");
console.log("🤖 STEP 2: Verifying Agent Instruction Templates");
console.log("--------------------------------------------------------------------------");

const expectedAgents = [
  "cursor-rules.md",
  "claude-code-instructions.md",
  "custom-agent-template.md"
];

for (const agent of expectedAgents) {
  const agentPath = path.join(__dirname, "agents", agent);
  if (fs.existsSync(agentPath)) {
    const stat = fs.statSync(agentPath);
    if (stat.size > 100) {
      pass(`Agent template present: ${agent} (${stat.size} bytes)`);
    } else {
      fail(`Agent template too small: ${agent}`);
    }
  } else {
    fail(`Missing agent template: ${agent}`);
  }
}

// ── Summary ─────────────────────────────────────────────────────────────────
console.log("\n==========================================================================");
console.log("📊 HYPERSPACEDB-SKILLS VALIDATION RESULTS");
console.log("==========================================================================");
console.log(`   ✅ PASSED: ${passed}`);
console.log(`   ❌ FAILED: ${failed}`);
console.log(`   Total:    ${passed + failed}`);
console.log("==========================================================================");

if (failed > 0) {
  process.exit(1);
} else {
  console.log("🎉 ALL 6 SKILLS AND 3 AGENT GUIDES VALIDATED 100%!\n");
  process.exit(0);
}
