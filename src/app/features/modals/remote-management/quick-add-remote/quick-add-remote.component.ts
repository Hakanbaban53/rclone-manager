import { Component, HostListener, OnInit, OnDestroy, inject } from '@angular/core';
import {
  AbstractControl,
  FormBuilder,
  FormGroup,
  ReactiveFormsModule,
  ValidationErrors,
  Validators,
} from '@angular/forms';

import { MatDialogRef } from '@angular/material/dialog';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatSelectModule } from '@angular/material/select';
import { MatInputModule } from '@angular/material/input';
import { MatDividerModule } from '@angular/material/divider';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatIconModule } from '@angular/material/icon';
import { MatExpansionModule } from '@angular/material/expansion';
import { Subscription } from 'rxjs';
import { MatButtonModule } from '@angular/material/button';
import { MatCheckboxModule } from '@angular/material/checkbox';
import {
  QuickAddForm,
  RemoteSettings,
  RemoteType,
} from '../../../../shared/remote-config/remote-config-types';

// Services
import { AnimationsService } from '../../../../shared/services/animations.service';
import { AuthStateService } from '../../../../shared/services/auth-state.service';
import { RemoteManagementService } from '@app/services';
import { JobManagementService } from '@app/services';
import { MountManagementService } from '@app/services';
import { AppSettingsService } from '@app/services';
import { FileSystemService } from '@app/services';

@Component({
  selector: 'app-quick-add-remote',
  standalone: true,
  imports: [
    ReactiveFormsModule,
    MatFormFieldModule,
    MatInputModule,
    MatSelectModule,
    MatDividerModule,
    MatCheckboxModule,
    MatProgressSpinnerModule,
    MatIconModule,
    MatButtonModule,
    MatExpansionModule,
  ],
  templateUrl: './quick-add-remote.component.html',
  styleUrls: ['./quick-add-remote.component.scss', '../../../../styles/_shared-modal.scss'],
  animations: [AnimationsService.slideInOut()],
})
export class QuickAddRemoteComponent implements OnInit, OnDestroy {
  quickAddForm: FormGroup;
  remoteTypes: RemoteType[] = [];
  existingRemotes: string[] = [];

  isAuthInProgress = false;
  isAuthCancelled = false;

  private subscriptions: Subscription[] = [];

  private fb = inject(FormBuilder);
  private dialogRef = inject(MatDialogRef<QuickAddRemoteComponent>);
  private authStateService = inject(AuthStateService);
  private remoteManagementService = inject(RemoteManagementService);
  private jobManagementService = inject(JobManagementService);
  private mountManagementService = inject(MountManagementService);
  private appSettingsService = inject(AppSettingsService);
  private fileSystemService = inject(FileSystemService);

  constructor() {
    this.quickAddForm = this.createQuickAddForm();
    this.setupFormListeners();
  }

  ngOnInit(): void {
    this.initializeComponent();
    this.setupAuthStateListeners();
  }

  private setupAuthStateListeners(): void {
    this.subscriptions.push(
      this.authStateService.isAuthInProgress$.subscribe(isInProgress => {
        this.isAuthInProgress = isInProgress;
        this.setFormState(isInProgress);
      })
    );
    this.subscriptions.push(
      this.authStateService.isAuthCancelled$.subscribe(isCancelled => {
        this.isAuthCancelled = isCancelled;
        console.log('Auth cancelled:', isCancelled);
      })
    );
  }

  ngOnDestroy(): void {
    this.cleanupSubscriptions();
    this.cleanup();
  }

  private cleanupSubscriptions(): void {
    this.subscriptions.forEach(sub => sub.unsubscribe());
    this.subscriptions = [];
  }

  private createQuickAddForm(): FormGroup {
    return this.fb.group({
      remoteName: ['', [Validators.required, this.validateRemoteName.bind(this)]],
      remoteType: ['', Validators.required],
      // Mount options
      autoMount: [false],
      mountSource: [''],
      mountPath: [''],
      // Sync options
      autoSync: [false],
      syncSource: [''],
      syncDest: [''],
      // Copy options
      autoCopy: [false],
      copySource: [''],
      copyDest: [''],
    });
  }

  private setupFormListeners(): void {
    // Mount path validation - only destination required
    const autoMountSub = this.quickAddForm
      .get('autoMount')
      ?.valueChanges.subscribe((autoMount: boolean) => {
        const mountPathControl = this.quickAddForm.get('mountPath');
        if (autoMount) {
          mountPathControl?.setValidators([Validators.required]);
        } else {
          mountPathControl?.clearValidators();
        }
        mountPathControl?.updateValueAndValidity();
      });

    // Sync validation - only destination required
    const autoSyncSub = this.quickAddForm
      .get('autoSync')
      ?.valueChanges.subscribe((autoSync: boolean) => {
        const syncDestControl = this.quickAddForm.get('syncDest');

        if (autoSync) {
          syncDestControl?.setValidators([Validators.required]);
        } else {
          syncDestControl?.clearValidators();
        }

        syncDestControl?.updateValueAndValidity();
      });

    // Copy validation - only destination required
    const autoCopySub = this.quickAddForm
      .get('autoCopy')
      ?.valueChanges.subscribe((autoCopy: boolean) => {
        const copyDestControl = this.quickAddForm.get('copyDest');

        if (autoCopy) {
          copyDestControl?.setValidators([Validators.required]);
        } else {
          copyDestControl?.clearValidators();
        }

        copyDestControl?.updateValueAndValidity();
      });

    if (autoMountSub) this.subscriptions.push(autoMountSub);
    if (autoSyncSub) this.subscriptions.push(autoSyncSub);
    if (autoCopySub) this.subscriptions.push(autoCopySub);
  }

  private async initializeComponent(): Promise<void> {
    try {
      const oauthSupportedRemotes = await this.remoteManagementService.getOAuthSupportedRemotes();
      console.log('OAuth Supported Remotes:', oauthSupportedRemotes);
      this.remoteTypes = oauthSupportedRemotes.map(remote => ({
        value: remote.name,
        label: remote.description,
      }));
      this.existingRemotes = await this.remoteManagementService.getRemotes();
      console.log('OAuth Supported Remotes:', this.remoteTypes);
    } catch (error) {
      console.error('Error initializing component:', error);
    }
  }

  validateRemoteName(control: AbstractControl): ValidationErrors | null {
    const value = control.value?.trim();
    if (!value) return null;
    return this.existingRemotes.includes(value) ? { nameTaken: true } : null;
  }

  onRemoteTypeChange(): void {
    const selectedRemote = this.quickAddForm.get('remoteType')?.value;
    if (!selectedRemote) return;

    const baseName = selectedRemote.replace(/\s+/g, '');
    let newName = baseName;
    let counter = 1;

    while (this.existingRemotes.includes(newName)) {
      newName = `${baseName}-${counter}`;
      counter++;
    }

    this.quickAddForm.patchValue({ remoteName: newName });
  }

  async selectFolder(fieldName = 'mountPath'): Promise<void> {
    try {
      const selectedPath = await this.fileSystemService.selectFolder(true);
      if (selectedPath) {
        this.quickAddForm.patchValue({ [fieldName]: selectedPath });
      }
    } catch (error) {
      console.error('Error selecting folder:', error);
    }
  }

  async onSubmit(): Promise<void> {
    if (this.quickAddForm.invalid || this.isAuthInProgress) return;

    const formValue = this.quickAddForm.value as QuickAddForm;
    await this.authStateService.startAuth(formValue.remoteName, false);

    try {
      await this.handleRemoteCreation(formValue);
      if (!this.isAuthCancelled) {
        this.dialogRef.close(true);
      }
    } catch (error) {
      console.error('Error in onSubmit:', error);
    } finally {
      this.authStateService.resetAuthState();
    }
  }

  private async handleRemoteCreation(formValue: QuickAddForm): Promise<void> {
    const { remoteName, remoteType, autoMount, mountPath, autoSync, syncDest, autoCopy, copyDest } =
      formValue;

    await this.remoteManagementService.createRemote(remoteName, {
      name: remoteName,
      type: remoteType,
    });

    const remoteSettings: RemoteSettings = {
      name: remoteName,
      vfsConfig: { CacheMode: 'full', ChunkSize: '32M' },
      showOnTray: true,
      mountConfig: {
        dest: mountPath || '',
        source: remoteName + ':/',
        autoStart: autoMount || false,
        type: 'mount',
      },
      copyConfig: {
        autoStart: autoCopy || false,
        source: remoteName + ':/',
        dest: copyDest || '',
      },
      syncConfig: {
        autoStart: autoSync || false,
        source: remoteName + ':/',
        dest: syncDest || '',
      },
      filterConfig: {},
      moveConfig: {
        autoStart: false,
        source: remoteName + ':/',
        dest: '',
      },
      bisyncConfig: {
        autoStart: false,
        source: remoteName + ':/',
        dest: '',
      },
    };

    await this.appSettingsService.saveRemoteSettings(remoteName, remoteSettings);

    // Auto-start operations based on user selections
    if (autoMount && mountPath) {
      const finalMountSource = remoteName + ':/';
      await this.mountManagementService.mountRemote(remoteName, finalMountSource, mountPath);
      console.log('Remote mounted successfully!');
    }

    if (autoSync && syncDest) {
      // Note: You may need to implement auto-sync starting logic
      const finalSyncSource = remoteName + ':/';
      await this.jobManagementService.startSync(remoteName, finalSyncSource, syncDest);
      console.log('Auto-sync configured for:', { source: finalSyncSource, dest: syncDest });
    }

    if (autoCopy && copyDest) {
      // Note: You may need to implement auto-copy starting logic
      const finalCopySource = remoteName + ':/';
      await this.jobManagementService.startCopy(remoteName, finalCopySource, copyDest);
      console.log('Auto-copy configured for:', { source: finalCopySource, dest: copyDest });
    }
  }

  async cancelAuth(): Promise<void> {
    await this.authStateService.cancelAuth();
  }

  private setFormState(disabled: boolean): void {
    if (disabled) {
      this.quickAddForm.disable();
    } else {
      this.quickAddForm.enable();
    }
  }

  getSubmitButtonText(): string {
    if (this.isAuthInProgress && !this.isAuthCancelled) {
      return 'Adding Remote...';
    }
    return 'Create Remote';
  }

  onPanelToggle(operation: 'mount' | 'sync' | 'copy', isOpen: boolean): void {
    const controlName = `auto${operation.charAt(0).toUpperCase() + operation.slice(1)}`;
    this.quickAddForm.patchValue({ [controlName]: isOpen });
  }

  @HostListener('document:keydown.escape', ['$event'])
  close(): void {
    this.dialogRef.close();
  }

  private cleanup(): void {
    this.subscriptions.forEach(sub => sub.unsubscribe());
    this.cancelAuth();
  }
}
