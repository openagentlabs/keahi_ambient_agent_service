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
    <div className="min-h-screen flex items-center justify-center bg-blue-900/60">
      <button
        className="px-3 py-1 text-sm rounded bg-blue-600 text-white shadow hover:bg-blue-700 transition"
        onClick={() => setCount((c) => c + 1)}
      >
        Click
      </button>
    </div>
  );
}
