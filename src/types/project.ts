export interface Project {
  id: string;
  name: string;
  path: string;
  created_at: string;
  updated_at: string;
}

export interface ProjectSummary {
  id: string;
  name: string;
  path: string;
  updated_at: string;
}

export interface RecentProject {
  name: string;
  path: string;
  last_opened: string;
}
