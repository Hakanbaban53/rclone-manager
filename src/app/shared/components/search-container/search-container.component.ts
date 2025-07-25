import { Component, Input, Output, EventEmitter, ViewChild, ElementRef } from '@angular/core';

import { FormsModule } from '@angular/forms';
import { MatInputModule } from '@angular/material/input';

// Services
import { AnimationsService } from '../../services/animations.service';

@Component({
  selector: 'app-search-container',
  standalone: true,
  imports: [FormsModule, MatInputModule],
  animations: [AnimationsService.slideToggle()],
  template: `
    <div class="search-container" [@slideToggle]="visible ? 'visible' : 'hidden'">
      <input
        #searchInput
        matInput
        [(ngModel)]="searchText"
        (ngModelChange)="onSearchTextChange($event)"
        [placeholder]="placeholder"
        [attr.aria-label]="ariaLabel"
        class="search-input"
      />
    </div>
  `,
  styleUrls: ['./search-container.component.scss'],
})
export class SearchContainerComponent {
  @Input() visible = false;
  @Input() placeholder = 'Search...';
  @Input() ariaLabel = 'Search';
  @Input() searchText = '';

  @Output() searchTextChange = new EventEmitter<string>();
  @Output() visibilityChange = new EventEmitter<boolean>();

  @ViewChild('searchInput') searchInput!: ElementRef<HTMLInputElement>;

  onSearchTextChange(value: string): void {
    this.searchText = value;
    this.searchTextChange.emit(value);
  }

  focus(): void {
    // Focus after animation completes
    setTimeout(() => {
      if (this.searchInput?.nativeElement) {
        this.searchInput.nativeElement.focus();
      }
    }, 300);
  }

  clear(): void {
    this.searchText = '';
    this.onSearchTextChange('');
  }
}
