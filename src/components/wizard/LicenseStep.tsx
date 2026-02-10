import { useWizardStore } from "../../stores/wizardStore";
import { Button } from "../ui/Button";

export function LicenseStep() {
  const {
    licenseAccepted,
    privacyAccepted,
    setLicenseAccepted,
    setPrivacyAccepted,
    nextStep,
    prevStep,
  } = useWizardStore();

  return (
    <div>
      <h2 className="text-xl font-bold mb-2 text-[var(--text-primary)]">
        License & Privacy
      </h2>
      <p className="text-sm text-[var(--text-secondary)] mb-6">
        Please review and accept the following agreements to continue.
      </p>

      <div className="bg-surface-50 dark:bg-surface-900 border border-[var(--border-color)] rounded-lg p-4 mb-4 max-h-40 overflow-y-auto text-xs text-[var(--text-secondary)] leading-relaxed">
        <p className="font-medium mb-2">Citinet End User License Agreement</p>
        <p>
          By installing and using Citinet, you agree to participate in a
          decentralized cloud network. Your device will contribute storage and
          bandwidth resources as configured. All data stored on the network is
          encrypted end-to-end. You retain full control over your contribution
          levels and can opt out at any time.
        </p>
        <p className="mt-2">
          Citinet is open-source software distributed under the MIT License.
          The software is provided "as is" without warranty of any kind.
        </p>
      </div>

      <label className="flex items-center gap-3 mb-3 cursor-pointer">
        <input
          type="checkbox"
          checked={licenseAccepted}
          onChange={(e) => setLicenseAccepted(e.target.checked)}
          className="w-4 h-4 rounded border-surface-300 text-primary-500 focus:ring-primary-500"
        />
        <span className="text-sm text-[var(--text-primary)]">
          I accept the License Agreement
        </span>
      </label>

      <label className="flex items-center gap-3 mb-8 cursor-pointer">
        <input
          type="checkbox"
          checked={privacyAccepted}
          onChange={(e) => setPrivacyAccepted(e.target.checked)}
          className="w-4 h-4 rounded border-surface-300 text-primary-500 focus:ring-primary-500"
        />
        <span className="text-sm text-[var(--text-primary)]">
          I accept the Privacy Policy
        </span>
      </label>

      <div className="flex gap-3">
        <Button variant="secondary" onClick={prevStep} className="flex-1">
          Back
        </Button>
        <Button
          onClick={nextStep}
          disabled={!licenseAccepted || !privacyAccepted}
          className="flex-1"
        >
          Continue
        </Button>
      </div>
    </div>
  );
}
