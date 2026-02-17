import { useWizardStore } from "../../stores/wizardStore";
import { WizardLayout } from "./WizardLayout";
import { WelcomeStep } from "./WelcomeStep";
import { LicenseStep } from "./LicenseStep";
import { NodeIdentityStep } from "./NodeIdentityStep";
import { LocationStep } from "./LocationStep";
import { ContributionSlider } from "./ContributionSlider";
import { ServiceStep } from "./ServiceStep";
import { TunnelStep } from "./TunnelStep";
import { AdminAccountStep } from "./AdminAccountStep";
import { ProgressStep } from "./ProgressStep";
import { CompleteStep } from "./CompleteStep";

const STEPS = [
  WelcomeStep,
  LicenseStep,
  NodeIdentityStep,
  LocationStep,
  ContributionSlider,
  ServiceStep,
  AdminAccountStep,
  ProgressStep,
  TunnelStep,
  CompleteStep,
];

export function Wizard() {
  const currentStep = useWizardStore((s) => s.currentStep);
  const StepComponent = STEPS[currentStep];

  return (
    <WizardLayout>
      <StepComponent />
    </WizardLayout>
  );
}
