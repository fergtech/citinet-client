import { create } from "zustand";
import type { User } from "../api/tauri";

interface AuthState {
  currentUser: User | null;
  setCurrentUser: (user: User) => void;
  logout: () => void;
}

const stored = localStorage.getItem("citinet-auth-user");
const initialUser: User | null = stored ? JSON.parse(stored) : null;

export const useAuthStore = create<AuthState>((set) => ({
  currentUser: initialUser,
  setCurrentUser: (user) => {
    localStorage.setItem("citinet-auth-user", JSON.stringify(user));
    set({ currentUser: user });
  },
  logout: () => {
    localStorage.removeItem("citinet-auth-user");
    localStorage.setItem("citinet-phase", "login");
    set({ currentUser: null });
    // Force page reload to reset all state cleanly
    window.location.reload();
  },
}));
