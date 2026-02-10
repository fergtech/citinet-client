import { useWizardStore } from "../../stores/wizardStore";
import { WizardLayout } from "./WizardLayout";
import { WelcomeStep } from "./WelcomeStep";
import { LicenseStep } from "./LicenseStep";
import { LocationStep } from "./LocationStep";
import { ServiceStep } from "./ServiceStep";
import { ProgressStep } from "./ProgressStep";
import { SuccessStep } from "./SuccessStep";

const STEPS = [
  WelcomeStep,
  LicenseStep,
  LocationStep,
  ServiceStep,
  ProgressStep,
  SuccessStep,
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
