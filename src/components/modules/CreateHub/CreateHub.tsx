import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import Button from "../../ui/Button";
import { Plus, Save, Trash2, Play, FileCode } from "lucide-react";
import { useErrorHandler } from "@/utils/errorHandler";

interface Project {
  id: number;
  name: string;
  project_type: string;
  content: string;
  created_at: number;
  updated_at: number;
}

export default function CreateHub() {
  const errorHandler = useErrorHandler();
  const [projects, setProjects] = useState<Project[]>([]);
  const [selectedProject, setSelectedProject] = useState<Project | null>(null);
  const [projectName, setProjectName] = useState("");
  const [projectType, setProjectType] = useState("playground");
  const [projectContent, setProjectContent] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadProjects();
  }, []);

  const loadProjects = async () => {
    try {
      const data = await invoke<Project[]>("list_projects", { projectType: null });
      setProjects(data);
      if (data.length > 0 && !selectedProject) {
        setSelectedProject(data[0]);
        setProjectName(data[0].name);
        setProjectContent(data[0].content);
        setProjectType(data[0].project_type);
      }
      setLoading(false);
    } catch (error) {
      errorHandler.showError("Failed to load projects", error);
      setLoading(false);
    }
  };

  const handleCreateProject = async () => {
    if (!projectName.trim()) {
      errorHandler.showError("Please enter a project name");
      return;
    }

    try {
      await invoke("create_project", {
        name: projectName,
        projectType: projectType,
        content: projectContent || getDefaultContent(projectType),
      });
      setProjectName("");
      setProjectContent("");
      await loadProjects();
      errorHandler.showSuccess("Project created successfully");
    } catch (error) {
      errorHandler.showError("Failed to create project", error);
    }
  };

  const handleUpdateProject = async () => {
    if (!selectedProject) return;

    try {
      await invoke("update_project", {
        id: selectedProject.id,
        name: projectName,
        content: projectContent,
      });
      await loadProjects();
      errorHandler.showSuccess("Project updated successfully");
    } catch (error) {
      errorHandler.showError("Failed to update project", error);
    }
  };

  const handleDeleteProject = async (id: number) => {
    if (!confirm("Are you sure you want to delete this project?")) return;

    try {
      await invoke("delete_project", { id });
      if (selectedProject?.id === id) {
        setSelectedProject(null);
        setProjectName("");
        setProjectContent("");
      }
      await loadProjects();
      errorHandler.showSuccess("Project deleted successfully");
    } catch (error) {
      errorHandler.showError("Failed to delete project", error);
    }
  };

  const handleSelectProject = async (id: number) => {
    try {
      const project = await invoke<Project | null>("get_project", { id });
      if (project) {
        setSelectedProject(project);
        setProjectName(project.name);
        setProjectContent(project.content);
        setProjectType(project.project_type);
      }
    } catch (error) {
      errorHandler.showError("Failed to load project", error);
    }
  };

  const getDefaultContent = (type: string) => {
    switch (type) {
      case "playground":
        return `// p5.js Playground
function setup() {
  createCanvas(400, 400);
}

function draw() {
  background(0);
  fill(0, 255, 255);
  ellipse(mouseX, mouseY, 50, 50);
}`;
      case "shader":
        return `// GLSL Fragment Shader
#ifdef GL_ES
precision mediump float;
#endif

uniform float u_time;
uniform vec2 u_resolution;

void main() {
  vec2 st = gl_FragCoord.xy / u_resolution.xy;
  vec3 color = vec3(st.x, st.y, sin(u_time));
  gl_FragColor = vec4(color, 1.0);
}`;
      case "script":
        return `// JavaScript/TypeScript Script
console.log("Hello from MINA!");

// Your code here
function greet(name) {
  return \`Hello, \${name}!\`;
}

greet("World");`;
      default:
        return "// Your code here";
    }
  };

  if (loading) {
    return <div className="text-center">Loading...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
          Create Hub
        </h1>
        <p className="text-gray-400">Creative coding and experimentation</p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-1">
          <Card title="Projects">
            <div className="space-y-2">
              <Button variant="primary" className="w-full" onClick={handleCreateProject}>
                <Plus className="w-4 h-4 mr-2" />
                New Project
              </Button>
              <div className="space-y-1 max-h-96 overflow-y-auto">
                {projects.map((project) => (
                  <button
                    key={project.id}
                    onClick={() => handleSelectProject(project.id)}
                    className={`w-full text-left p-2 rounded glass-card transition-all flex items-center justify-between ${
                      selectedProject?.id === project.id
                        ? "border-2 border-neon-cyan"
                        : "hover:border border-white/10"
                    }`}
                  >
                    <div className="flex-1 min-w-0">
                      <div className="font-semibold text-sm truncate">{project.name}</div>
                      <div className="text-xs text-gray-400">{project.project_type}</div>
                    </div>
                    <Button
                      variant="ghost"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDeleteProject(project.id);
                      }}
                    >
                      <Trash2 className="w-3 h-3 text-neon-red" />
                    </Button>
                  </button>
                ))}
              </div>
            </div>
          </Card>
        </div>

        <div className="lg:col-span-2">
          <Card title={selectedProject ? `Edit: ${selectedProject.name}` : "New Project"}>
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm text-gray-400 mb-2">Project Name</label>
                  <input
                    type="text"
                    value={projectName}
                    onChange={(e) => setProjectName(e.target.value)}
                    className="glass-input w-full"
                    placeholder="my_project"
                  />
                </div>
                <div>
                  <label className="block text-sm text-gray-400 mb-2">Project Type</label>
                  <select
                    value={projectType}
                    onChange={(e) => {
                      setProjectType(e.target.value);
                      if (!selectedProject) {
                        setProjectContent(getDefaultContent(e.target.value));
                      }
                    }}
                    className="glass-input w-full"
                    disabled={!!selectedProject}
                  >
                    <option value="playground">Playground (p5.js)</option>
                    <option value="shader">Shader (GLSL)</option>
                    <option value="script">Script (JS/TS)</option>
                    <option value="game">Game</option>
                  </select>
                </div>
              </div>
              <div>
                <label className="block text-sm text-gray-400 mb-2">Code</label>
                <textarea
                  value={projectContent}
                  onChange={(e) => setProjectContent(e.target.value)}
                  className="glass-input w-full font-mono text-sm"
                  rows={25}
                  placeholder="// Your code here"
                />
              </div>
              <div className="flex gap-2">
                {selectedProject ? (
                  <>
                    <Button variant="primary" onClick={handleUpdateProject}>
                      <Save className="w-4 h-4 mr-2" />
                      Save Changes
                    </Button>
                    <Button variant="secondary">
                      <Play className="w-4 h-4 mr-2" />
                      Run
                    </Button>
                  </>
                ) : (
                  <Button variant="primary" onClick={handleCreateProject}>
                    <Plus className="w-4 h-4 mr-2" />
                    Create Project
                  </Button>
                )}
              </div>
            </div>
          </Card>

          {selectedProject && projectType === "playground" && (
            <Card title="Preview" className="mt-6">
              <div className="bg-black rounded p-4 min-h-[300px]">
                <div className="text-center text-gray-400 py-8">
                  <FileCode className="w-12 h-12 mx-auto mb-4 text-gray-500" />
                  <p>Preview pane for playground projects</p>
                  <p className="text-xs mt-2">In production, this would render p5.js canvas</p>
                </div>
              </div>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
}
