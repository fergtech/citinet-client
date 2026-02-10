import { Button } from "../ui/Button";
import { useAppStore } from "../../stores/appStore";
import { Rocket } from "lucide-react";

export function ReadyPanel() {
  const setPhase = useAppStore((s) => s.setPhase);

  return (
    <div className="text-center">
      <Rocket className="w-16 h-16 text-primary-500 mx-auto mb-4" />
      <h2 className="text-2xl font-bold mb-2 text-[var(--text-primary)]">
        You're All Set!
      </h2>
      <p className="text-[var(--text-secondary)] mb-8">
        Your Citinet node is configured and ready to join the network.
        Welcome to the people-powered cloud.
      </p>

      <Button
        onClick={() => setPhase("dashboard")}
        size="lg"
        className="w-full"
      >
        Open Dashboard
      </Button>
    </div>
  );
}
