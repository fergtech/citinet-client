import { useOnboardingStore } from "../../stores/onboardingStore";
import { OnboardingLayout } from "./OnboardingLayout";
import { WelcomePanel } from "./WelcomePanel";
import { CreateProfile } from "./CreateProfile";
import { IdentityKeys } from "./IdentityKeys";
import { ContributionSlider } from "./ContributionSlider";
import { CommunityPanel } from "./CommunityPanel";
import { NetworkCheck } from "./NetworkCheck";
import { ReadyPanel } from "./ReadyPanel";

const STEPS = [
  WelcomePanel,
  CreateProfile,
  IdentityKeys,
  ContributionSlider,
  CommunityPanel,
  NetworkCheck,
  ReadyPanel,
];

export function Onboarding() {
  const currentStep = useOnboardingStore((s) => s.currentStep);
  const StepComponent = STEPS[currentStep];

  return (
    <OnboardingLayout>
      <StepComponent />
    </OnboardingLayout>
  );
}
