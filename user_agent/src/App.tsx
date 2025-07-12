import * as React from "react";
import { Button } from "@/components/ui/button";
import { invoke } from "@tauri-apps/api/core";

interface SystemInfo {
  os: string;
  arch: string;
  hostname: string;
  username: string;
}

interface UserAgentInfo {
  name: string;
  version: string;
  capabilities: string[];
}

export default function App() {
  const [count, setCount] = React.useState(0);
  const [systemInfo, setSystemInfo] = React.useState<SystemInfo | null>(null);
  const [userAgentInfo, setUserAgentInfo] = React.useState<UserAgentInfo | null>(null);
  const [commandOutput, setCommandOutput] = React.useState<string>("");
  const [loading, setLoading] = React.useState(false);

  const getSystemInfo = async () => {
    setLoading(true);
    try {
      const info = await invoke<SystemInfo>("get_system_info");
      setSystemInfo(info);
    } catch (error) {
      console.error("Failed to get system info:", error);
    }
    setLoading(false);
  };

  const getUserAgentInfo = async () => {
    setLoading(true);
    try {
      const info = await invoke<UserAgentInfo>("get_user_agent_info");
      setUserAgentInfo(info);
    } catch (error) {
      console.error("Failed to get user agent info:", error);
    }
    setLoading(false);
  };

  const executeCommand = async () => {
    setLoading(true);
    try {
      const output = await invoke<string>("execute_command", {
        command: "ls",
        args: ["-la"]
      });
      setCommandOutput(output);
    } catch (error) {
      console.error("Failed to execute command:", error);
    }
    setLoading(false);
  };

  return (
    <div className="container mx-auto p-8 flex flex-col items-center justify-center min-h-screen bg-background text-foreground">
      <div className="text-center mb-8">
        <h1 className="text-4xl font-bold mb-4">Keahi Ambient Agent</h1>
        <p className="text-lg text-muted-foreground mb-4">
          A Rust-powered desktop application with React UI
        </p>
        <p className="text-sm text-muted-foreground">
          This is primarily a Rust application using Tauri, with a React frontend for the UI
        </p>
      </div>

      {/* Centered Button */}
      <div className="flex justify-center items-center mb-8">
        <Button 
          size="lg" 
          className="text-lg px-8 py-4 h-auto"
          onClick={() => setCount((c) => c + 1)}
        >
          🦀 Rust Button - Clicked {count} times
        </Button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 w-full max-w-4xl">
        {/* System Information */}
        <div className="space-y-4">
          <h2 className="text-2xl font-semibold">System Information</h2>
          <Button 
            onClick={getSystemInfo} 
            disabled={loading}
            className="w-full"
          >
            {loading ? "Loading..." : "Get System Info"}
          </Button>
          
          {systemInfo && (
            <div className="bg-muted p-4 rounded-lg space-y-2">
              <p><strong>OS:</strong> {systemInfo.os}</p>
              <p><strong>Architecture:</strong> {systemInfo.arch}</p>
              <p><strong>Hostname:</strong> {systemInfo.hostname}</p>
              <p><strong>Username:</strong> {systemInfo.username}</p>
            </div>
          )}
        </div>

        {/* User Agent Information */}
        <div className="space-y-4">
          <h2 className="text-2xl font-semibold">Agent Information</h2>
          <Button 
            onClick={getUserAgentInfo} 
            disabled={loading}
            className="w-full"
          >
            {loading ? "Loading..." : "Get Agent Info"}
          </Button>
          
          {userAgentInfo && (
            <div className="bg-muted p-4 rounded-lg space-y-2">
              <p><strong>Name:</strong> {userAgentInfo.name}</p>
              <p><strong>Version:</strong> {userAgentInfo.version}</p>
              <p><strong>Capabilities:</strong></p>
              <ul className="list-disc list-inside ml-4">
                {userAgentInfo.capabilities.map((cap, index) => (
                  <li key={index}>{cap}</li>
                ))}
              </ul>
            </div>
          )}
        </div>

        {/* Command Execution */}
        <div className="space-y-4 md:col-span-2">
          <h2 className="text-2xl font-semibold">Rust Command Execution</h2>
          <Button 
            onClick={executeCommand} 
            disabled={loading}
            className="w-full"
          >
            {loading ? "Executing..." : "Execute 'ls -la' (Rust)"}
          </Button>
          
          {commandOutput && (
            <div className="bg-muted p-4 rounded-lg">
              <p className="font-mono text-sm whitespace-pre-wrap">{commandOutput}</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
