export const uiTheme = {
  colors: {
    neutral: 'stone',
    primary: 'moss'
  },

  button: {
    slots: {
      base: 'twill-control justify-center'
    },

    variants: {
      size: {
        xs: {
          base: 'min-h-6 px-1.5 py-0.5',
          leadingIcon: 'size-3.5',
          trailingIcon: 'size-3.5'
        },

        sm: {
          base: 'min-h-6 px-2 py-0.5',
          leadingIcon: 'size-3.5',
          trailingIcon: 'size-3.5'
        },

        md: {
          base: 'min-h-7 px-2 py-1',
          leadingIcon: 'size-3.5',
          trailingIcon: 'size-3.5'
        },

        lg: {
          base: 'min-h-8 px-2.5 py-1.5',
          leadingIcon: 'size-4',
          trailingIcon: 'size-4'
        },

        xl: {
          base: 'min-h-9 px-3 py-1.5',
          leadingIcon: 'size-4',
          trailingIcon: 'size-4'
        }
      }
    },

    defaultVariants: {
      variant: 'subtle'
    }
  },

  input: {
    slots: { base: 'twill-field' },
    defaultVariants: { variant: 'subtle' }
  },

  select: {
    slots: { base: 'twill-field', content: 'z-80' },
    defaultVariants: { variant: 'subtle' }
  },

  textarea: {
    defaultVariants: { variant: 'subtle' }
  },

  modal: {
    slots: {
      overlay: 'z-70',
      content: 'z-71',
      header: 'min-h-0 p-4 sm:px-4',
      body: 'p-4 sm:p-4',
      footer: 'p-4 sm:px-4',
      title: 'text-base',
      close: 'top-3 end-3'
    }
  }
};
