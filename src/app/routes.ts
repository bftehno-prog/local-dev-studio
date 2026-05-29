import { Activity, AppWindow, Boxes, Folder, Gauge, Globe2, Settings as SettingsIcon, Terminal } from "lucide-react";

export const sections = [
  ["dashboard", "nav.dashboard", Gauge],
  ["projects", "nav.projects", Folder],
  ["sandboxes", "nav.sandboxes", Boxes],
  ["templates", "nav.templates", AppWindow],
  ["servers", "nav.servers", Activity],
  ["ports", "nav.ports", Globe2],
  ["logs", "nav.logs", Terminal],
  ["settings", "nav.settings", SettingsIcon]
] as const;

export type SectionId = (typeof sections)[number][0];

export const devices = [
  ["Desktop", 1440],
  ["Laptop", 1280],
  ["Tablet", 768],
  ["Mobile", 390],
  ["Custom", 980]
] as const;
